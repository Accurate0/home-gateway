use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Australia::Perth;
use std::collections::{BTreeSet, HashMap};

use crate::repo::light::ProfileBucket;
use crate::settings::VacationSettings;

pub const SLOTS_PER_DAY: i16 = 48;
const SLOT_MINUTES: i64 = 30;

const THRESHOLD_LOW: f64 = 0.35;
const THRESHOLD_HIGH: f64 = 0.65;
const HYSTERESIS: f64 = 0.1;
const MIN_RUN_SLOTS: usize = 2;

const JITTER_SALT: u64 = 1;
const THRESHOLD_SALT: u64 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedAction {
    pub address: String,
    pub at: DateTime<Utc>,
    pub on: bool,
    pub slot: i16,
    pub on_fraction: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DayPlan {
    pub initial: HashMap<String, bool>,
    pub actions: Vec<PlannedAction>,
    pub closing: HashMap<String, bool>,
}

impl DayPlan {
    pub fn addresses(&self) -> BTreeSet<&str> {
        self.initial
            .keys()
            .map(String::as_str)
            .chain(self.actions.iter().map(|action| action.address.as_str()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct Run {
    value: Option<bool>,
    start: usize,
    len: usize,
}

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);

    value ^ (value >> 31)
}

fn address_seed(address: &str) -> u64 {
    address.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

/// A deterministic draw in `[0, 1)` for one (light, day, slot, salt). The plan is
/// therefore a pure function of its inputs, so the API and the actor agree and a
/// restart mid-day resumes the same plan rather than re-rolling it.
fn draw(settings: &VacationSettings, address: &str, day: NaiveDate, slot: i16, salt: u64) -> f64 {
    let seed = mix(settings.seed)
        ^ mix(address_seed(address))
        ^ mix(day
            .and_hms_opt(0, 0, 0)
            .map_or(0, |d| d.and_utc().timestamp()) as u64)
        ^ mix(slot as u64)
        ^ mix(salt);

    mix(seed) as f64 / u64::MAX as f64
}

fn slot_start(day: NaiveDate, slot: i16) -> DateTime<Utc> {
    let local = day.and_hms_opt(0, 0, 0).expect("midnight exists")
        + Duration::minutes(i64::from(slot) * SLOT_MINUTES);

    Perth
        .from_local_datetime(&local)
        .earliest()
        .unwrap_or_else(|| Perth.from_utc_datetime(&local))
        .with_timezone(&Utc)
}

fn profiles<'a>(
    buckets: &'a [ProfileBucket],
    day: NaiveDate,
    settings: &VacationSettings,
) -> HashMap<&'a str, HashMap<i16, &'a ProfileBucket>> {
    let isodow = day.weekday().number_from_monday() as i16;
    let mut by_address: HashMap<&str, HashMap<i16, &ProfileBucket>> = HashMap::new();

    for bucket in buckets {
        if bucket.isodow != isodow || bucket.observations < settings.min_observations {
            continue;
        }

        by_address
            .entry(bucket.address.as_str())
            .or_default()
            .insert(bucket.slot, bucket);
    }

    by_address
}

fn day_targets(
    settings: &VacationSettings,
    address: &str,
    day: NaiveDate,
    slots: Option<&HashMap<i16, &ProfileBucket>>,
    initial: Option<bool>,
) -> Vec<Option<bool>> {
    let threshold = THRESHOLD_LOW
        + (THRESHOLD_HIGH - THRESHOLD_LOW) * draw(settings, address, day, 0, THRESHOLD_SALT);

    let mut state = initial;
    let mut targets = Vec::with_capacity(SLOTS_PER_DAY as usize);

    for slot in 0..SLOTS_PER_DAY {
        if let Some(bucket) = slots.and_then(|slots| slots.get(&slot)) {
            let fraction = bucket.on_fraction;

            state = Some(match state {
                Some(true) => fraction >= threshold - HYSTERESIS,
                Some(false) => fraction >= threshold + HYSTERESIS,
                None => fraction >= threshold,
            });
        }

        targets.push(state);
    }

    smooth(&mut targets);

    targets
}

fn smooth(targets: &mut [Option<bool>]) {
    let mut runs: Vec<Run> = Vec::new();

    for (index, target) in targets.iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.value == *target => run.len += 1,
            _ => runs.push(Run {
                value: *target,
                start: index,
                len: 1,
            }),
        }
    }

    for index in 1..runs.len().saturating_sub(1) {
        let run = runs[index];
        let fill = runs[index - 1].value;

        if run.value.is_none() || fill.is_none() || run.len >= MIN_RUN_SLOTS {
            continue;
        }

        targets[run.start..run.start + run.len].fill(fill);
        runs[index].value = fill;
    }
}

pub fn coverage(buckets: &[ProfileBucket], address: &str, day: NaiveDate, min: i64) -> usize {
    let isodow = day.weekday().number_from_monday() as i16;

    buckets
        .iter()
        .filter(|bucket| {
            bucket.address == address && bucket.isodow == isodow && bucket.observations >= min
        })
        .count()
}

fn plan_day(
    buckets: &[ProfileBucket],
    day: NaiveDate,
    settings: &VacationSettings,
    initial: &HashMap<String, bool>,
) -> DayPlan {
    let profiles = profiles(buckets, day, settings);
    let addresses: BTreeSet<&str> = profiles
        .keys()
        .copied()
        .chain(initial.keys().map(String::as_str))
        .collect();

    let jitter_seconds = settings.jitter.num_seconds().max(0);
    let mut plan = DayPlan::default();

    for address in addresses {
        let slots = profiles.get(address);
        let start = initial.get(address).copied();
        let mut last = start;

        for (slot, target) in day_targets(settings, address, day, slots, start)
            .into_iter()
            .enumerate()
        {
            let slot = slot as i16;

            let Some(on) = target else {
                continue;
            };

            if last == Some(on) {
                continue;
            }

            last = Some(on);

            let offset = (draw(settings, address, day, slot, JITTER_SALT) * 2.0 - 1.0)
                * jitter_seconds as f64;

            plan.actions.push(PlannedAction {
                address: address.to_owned(),
                at: slot_start(day, slot) + Duration::seconds(offset as i64),
                on,
                slot,
                on_fraction: slots
                    .and_then(|slots| slots.get(&slot))
                    .map_or(0.0, |bucket| bucket.on_fraction),
            });
        }

        if let Some(start) = start {
            plan.initial.insert(address.to_owned(), start);
        }

        if let Some(last) = last {
            plan.closing.insert(address.to_owned(), last);
        }
    }

    plan.actions
        .sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.address.cmp(&b.address)));

    plan
}

pub fn build_days(
    buckets: &[ProfileBucket],
    first: NaiveDate,
    days: usize,
    settings: &VacationSettings,
) -> Vec<DayPlan> {
    let previous = first.pred_opt().unwrap_or(first);
    let mut carried = plan_day(buckets, previous, settings, &HashMap::new()).closing;
    let mut plans = Vec::with_capacity(days);

    for day in first.iter_days().take(days) {
        let plan = plan_day(buckets, day, settings, &carried);
        carried = plan.closing.clone();
        plans.push(plan);
    }

    plans
}

pub fn build_plan(
    buckets: &[ProfileBucket],
    day: NaiveDate,
    settings: &VacationSettings,
) -> DayPlan {
    build_days(buckets, day, 1, settings)
        .into_iter()
        .next()
        .unwrap_or_default()
}

pub fn target_at(plan: &DayPlan, address: &str, now: DateTime<Utc>) -> Option<bool> {
    plan.actions
        .iter()
        .rfind(|action| action.address == address && action.at <= now)
        .map(|action| action.on)
        .or_else(|| plan.initial.get(address).copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;

    use crate::settings::enabled_state::EnabledState;
    use crate::workflows::mode::Mode;

    fn settings() -> VacationSettings {
        VacationSettings {
            state: EnabledState::Enabled,
            modes: vec![Mode::Vacation],
            window: TimeDelta::hours(672),
            jitter: TimeDelta::minutes(12),
            min_observations: 8,
            seed: 42,
        }
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 8).expect("valid date")
    }

    fn bucket_on(isodow: i16, slot: i16, on_fraction: f64, observations: i64) -> ProfileBucket {
        ProfileBucket {
            address: "0x001".to_owned(),
            isodow,
            slot,
            on_fraction,
            observations,
            turned_on: 0,
        }
    }

    fn bucket(slot: i16, on_fraction: f64, observations: i64) -> ProfileBucket {
        bucket_on(2, slot, on_fraction, observations)
    }

    #[test]
    fn an_empty_profile_plans_nothing() {
        assert_eq!(build_plan(&[], day(), &settings()), DayPlan::default());
    }

    #[test]
    fn an_always_on_light_switches_on_once() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 1.0, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.actions.len(), 1);
        assert!(plan.actions[0].on);
        assert_eq!(plan.actions[0].slot, 0);
    }

    #[test]
    fn the_plan_is_the_same_every_time() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 0.5, 30))
            .collect::<Vec<_>>();

        assert_eq!(
            build_plan(&buckets, day(), &settings()),
            build_plan(&buckets, day(), &settings())
        );
    }

    #[test]
    fn thin_slots_are_left_alone() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 1.0, if slot < 24 { 30 } else { 1 }))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.actions.len(), 1);
        assert_eq!(coverage(&buckets, "0x001", day(), 8), 24);
    }

    #[test]
    fn a_thin_slot_holds_the_state_instead_of_repeating_it() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 0.0, if slot == 20 { 1 } else { 30 }))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.actions.len(), 1);
        assert!(!plan.actions[0].on);
    }

    #[test]
    fn every_action_stays_within_the_jitter_of_its_slot() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 0.5, 30))
            .collect::<Vec<_>>();

        for action in build_plan(&buckets, day(), &settings()).actions {
            let drift = (action.at - slot_start(day(), action.slot))
                .num_seconds()
                .abs();

            assert!(drift <= settings().jitter.num_seconds(), "drifted {drift}s");
        }
    }

    #[test]
    fn a_flickering_profile_is_smoothed_into_runs() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, if slot % 2 == 0 { 0.2 } else { 0.8 }, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        for pair in plan.actions.windows(2) {
            let gap = (pair[1].slot - pair[0].slot) as usize;

            assert!(gap >= MIN_RUN_SLOTS, "a {gap} slot run survived: {pair:?}");
        }
    }

    #[test]
    fn values_hovering_around_the_threshold_do_not_flicker() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, if slot % 3 == 0 { 0.47 } else { 0.53 }, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.actions.len(), 1);
    }

    #[test]
    fn an_evening_session_becomes_one_on_and_one_off() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, if (36..42).contains(&slot) { 0.9 } else { 0.1 }, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());
        let switches = plan
            .actions
            .iter()
            .map(|action| (action.slot, action.on))
            .collect::<Vec<_>>();

        assert_eq!(switches, vec![(0, false), (36, true), (42, false)]);
    }

    #[test]
    fn the_next_day_continues_from_the_previous_one() {
        let buckets = (0..SLOTS_PER_DAY)
            .flat_map(|slot| [bucket_on(2, slot, 0.0, 30), bucket_on(3, slot, 0.0, 30)])
            .collect::<Vec<_>>();

        let plans = build_days(&buckets, day(), 2, &settings());

        assert_eq!(plans[0].actions.len(), 1);
        assert!(plans[1].actions.is_empty());
        assert_eq!(plans[1].initial.get("0x001"), Some(&false));
    }

    #[test]
    fn the_current_target_is_the_last_action_that_has_passed() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 1.0, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(
            target_at(&plan, "0x001", plan.actions[0].at - Duration::hours(1)),
            None
        );
        assert_eq!(target_at(&plan, "0x001", plan.actions[0].at), Some(true));
    }

    #[test]
    fn before_the_first_switch_the_carried_state_is_the_target() {
        let buckets = (0..SLOTS_PER_DAY)
            .flat_map(|slot| [bucket_on(1, slot, 1.0, 30), bucket_on(2, slot, 1.0, 30)])
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert!(plan.actions.is_empty());
        assert_eq!(target_at(&plan, "0x001", slot_start(day(), 0)), Some(true));
    }

    #[test]
    fn arming_between_transitions_still_has_a_target() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, if slot < 24 { 0.0 } else { 1.0 }, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.actions.len(), 2);
        assert!(!plan.actions[0].on);
        assert!(plan.actions[1].on);

        let midway = plan.actions[1].at + Duration::hours(4);

        assert_eq!(target_at(&plan, "0x001", midway), Some(true));
    }
}
