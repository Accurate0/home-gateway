use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Australia::Perth;
use std::collections::HashMap;

use crate::repo::light::ProfileBucket;
use crate::settings::VacationSettings;

pub const SLOTS_PER_DAY: i16 = 48;
const SLOT_MINUTES: i64 = 30;

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedAction {
    pub address: String,
    pub at: DateTime<Utc>,
    pub on: bool,
    pub slot: i16,
    pub on_fraction: f64,
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

/// The target states of one light across a day, indexed by slot: `Some(on)` where
/// history is dense enough to have an opinion, `None` where the light is left
/// alone.
fn slot_states(
    settings: &VacationSettings,
    address: &str,
    day: NaiveDate,
    buckets: &HashMap<i16, &ProfileBucket>,
) -> Vec<Option<(bool, f64)>> {
    (0..SLOTS_PER_DAY)
        .map(|slot| {
            let bucket = buckets.get(&slot)?;
            let on = draw(settings, address, day, slot, 0) < bucket.on_fraction;

            Some((on, bucket.on_fraction))
        })
        .collect()
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

/// Turn the historical profile into the day's switching plan: draw a target state
/// per 30 minute slot, keep only the transitions, and jitter each one so the
/// house does not switch on the half hour every night.
pub fn build_plan(
    buckets: &[ProfileBucket],
    day: NaiveDate,
    settings: &VacationSettings,
) -> Vec<PlannedAction> {
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

    let jitter_seconds = settings.jitter.num_seconds().max(0);
    let mut actions = Vec::new();

    for (address, slots) in by_address {
        let mut previous = None;

        for (slot, state) in slot_states(settings, address, day, &slots)
            .into_iter()
            .enumerate()
        {
            let slot = slot as i16;

            let Some((on, on_fraction)) = state else {
                previous = None;
                continue;
            };

            if previous == Some(on) {
                continue;
            }

            previous = Some(on);

            let offset =
                (draw(settings, address, day, slot, 1) * 2.0 - 1.0) * jitter_seconds as f64;
            let at = slot_start(day, slot) + Duration::seconds(offset as i64);

            actions.push(PlannedAction {
                address: address.to_owned(),
                at,
                on,
                slot,
                on_fraction,
            });
        }
    }

    actions.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.address.cmp(&b.address)));

    actions
}

/// The state a light should be in right now: the most recent action at or before
/// `now`, or `None` when the plan has not reached the light yet today.
pub fn target_at(actions: &[PlannedAction], address: &str, now: DateTime<Utc>) -> Option<bool> {
    actions
        .iter()
        .rfind(|action| action.address == address && action.at <= now)
        .map(|action| action.on)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;

    use crate::mode::Mode;

    fn settings() -> VacationSettings {
        VacationSettings {
            enabled: true,
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

    fn bucket(slot: i16, on_fraction: f64, observations: i64) -> ProfileBucket {
        ProfileBucket {
            address: "0x001".to_owned(),
            isodow: 2,
            slot,
            on_fraction,
            observations,
            turned_on: 0,
        }
    }

    #[test]
    fn an_empty_profile_plans_nothing() {
        assert_eq!(build_plan(&[], day(), &settings()), Vec::new());
    }

    #[test]
    fn an_always_on_light_switches_on_once() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 1.0, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.len(), 1);
        assert!(plan[0].on);
        assert_eq!(plan[0].slot, 0);
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

        assert_eq!(plan.len(), 1);
        assert_eq!(coverage(&buckets, "0x001", day(), 8), 24);
    }

    #[test]
    fn every_action_stays_within_the_jitter_of_its_slot() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 0.5, 30))
            .collect::<Vec<_>>();

        for action in build_plan(&buckets, day(), &settings()) {
            let drift = (action.at - slot_start(day(), action.slot))
                .num_seconds()
                .abs();

            assert!(drift <= settings().jitter.num_seconds(), "drifted {drift}s");
        }
    }

    #[test]
    fn the_current_target_is_the_last_action_that_has_passed() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, 1.0, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(
            target_at(&plan, "0x001", plan[0].at - Duration::hours(1)),
            None
        );
        assert_eq!(target_at(&plan, "0x001", plan[0].at), Some(true));
    }

    #[test]
    fn arming_between_transitions_still_has_a_target() {
        let buckets = (0..SLOTS_PER_DAY)
            .map(|slot| bucket(slot, if slot < 24 { 0.0 } else { 1.0 }, 30))
            .collect::<Vec<_>>();

        let plan = build_plan(&buckets, day(), &settings());

        assert_eq!(plan.len(), 2);
        assert!(!plan[0].on);
        assert!(plan[1].on);

        let midway = plan[1].at + Duration::hours(4);

        assert_eq!(target_at(&plan, "0x001", midway), Some(true));
    }
}
