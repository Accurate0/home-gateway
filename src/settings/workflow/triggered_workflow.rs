use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use super::condition::resolve_opt;
use super::{Condition, ReusableWorkflow, TriggerMatcher};
use crate::mode::Mode;
use crate::settings::DeviceAliases;
use crate::timedelta_format::option_time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Workflow {
    #[serde(flatten)]
    pub body: ReusableWorkflow,
    pub on: TriggerMatcher,
    pub modes: Vec<Mode>,
    #[serde(default)]
    pub when: Option<Condition>,
    #[serde(default, deserialize_with = "option_time_delta_from_str::deserialize")]
    #[schemars(with = "Option<String>")]
    pub cooldown: Option<TimeDelta>,
    #[serde(default, deserialize_with = "option_time_delta_from_str::deserialize")]
    #[schemars(with = "Option<String>")]
    pub delay: Option<TimeDelta>,
    #[serde(
        default,
        rename = "for",
        deserialize_with = "option_time_delta_from_str::deserialize"
    )]
    #[schemars(with = "Option<String>")]
    pub hold: Option<TimeDelta>,
}

impl std::ops::Deref for Workflow {
    type Target = ReusableWorkflow;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl Workflow {
    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        self.on.resolve_devices(devices)?;
        resolve_opt(&mut self.when, devices)?;

        self.body.resolve_devices(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::NotifyActionKind;
    use crate::settings::workflow::{ContextSource, Step};
    use config::{Config, File, FileFormat};

    fn parse(yaml: &str) -> Workflow {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Workflow>()
            .unwrap()
    }

    #[test]
    fn fuelwatch_trigger_parses_and_exposes_vars() {
        let workflow: Workflow = serde_yaml::from_str(
            r#"
name: Fill up tomorrow
on: { type: fuelwatch, change: tomorrow_lower, min_drop: 5 }
modes: [home]
run: []
"#,
        )
        .unwrap();

        let on = &workflow.on;

        assert_eq!(on.event_kind(), "fuelwatch");
        assert!(!on.supports_hold());
        assert_eq!(on.describe(), "fuelwatch(*) tomorrow_lower drop >= 5");
    }

    #[test]
    fn home_assistant_trigger_and_step_parse() {
        let workflow = parse(
            r#"
name: HA test
slug: ha-test
on: { type: home_assistant, entity_id: binary_sensor.front_door, state: "on" }
modes: [home]
run:
  - type: home_assistant
    call_service: light.turn_on
    data: { entity_id: light.hallway }
"#,
        );

        assert!(matches!(
            &workflow.on,
            TriggerMatcher::HomeAssistant { entity_id, state }
                if entity_id == "binary_sensor.front_door" && state.as_deref() == Some("on")
        ));

        assert!(matches!(
            &workflow.run[0],
            Step::HomeAssistant { call_service, .. } if call_service == "light.turn_on"
        ));
    }

    #[test]
    fn context_parses_and_exposes_vars() {
        let workflow = parse(
            r#"
name: Fuel test
slug: fuel-test
on: { type: cron, schedule: "0 13 * * TUE" }
modes: [home]
context: [fuelwatch]
run:
  - type: notify
    notify: { type: android_app }
    category: general
    message: "${fuelwatch.price | round(1)}c/L at ${fuelwatch.brand}"
"#,
        );

        assert_eq!(workflow.context, vec![ContextSource::Fuelwatch]);

        let scope =
            crate::variables::Scope::default().with("fuelwatch", ContextSource::Fuelwatch.shape());

        for (_, template) in workflow.run.iter().flat_map(Step::templates) {
            template.check(&scope).expect("fuelwatch template checks");
        }
    }

    #[test]
    fn notify_step_parses_category_and_actions() {
        let workflow = parse(
            r#"
name: Notify test
slug: notify-test
on: { type: presence, sensor: hallway, present: true }
modes: [home]
run:
  - type: notify
    notify: { type: android_app }
    category: alarm
    title: "Wake up"
    message: "Alarm in 5 minutes"
    actions:
      - label: Snooze
        action: { type: snooze, seconds: 600 }
      - label: Lights on
        action: { type: run_workflow, workflow: alarm-wakeup }
      - label: Dismiss
        action: { type: dismiss }
"#,
        );

        let Step::Notify {
            category,
            title,
            actions,
            ..
        } = &workflow.run[0]
        else {
            panic!("expected a notify step");
        };

        assert_eq!(category.as_str(), "alarm");
        assert_eq!(
            title.as_ref().map(ToString::to_string),
            Some("Wake up".to_string())
        );
        assert_eq!(actions.len(), 3);
        assert!(matches!(
            &actions[1].action,
            NotifyActionKind::RunWorkflow { workflow } if workflow == "alarm-wakeup"
        ));
        assert_eq!(workflow.notify_action_targets(), vec!["alarm-wakeup"]);
    }

    #[test]
    fn notify_step_requires_a_category() {
        let err = Config::builder()
            .add_source(File::from_str(
                r#"
name: Notify test
slug: notify-test
on: { type: presence, sensor: hallway, present: true }
modes: [home]
run:
  - type: notify
    notify: { type: android_app }
    message: "no category"
"#,
                FileFormat::Yaml,
            ))
            .build()
            .unwrap()
            .try_deserialize::<Workflow>()
            .unwrap_err();

        assert!(err.to_string().contains("category"), "{err}");
    }

    fn notify_workflow(step: &str) -> Workflow {
        let yaml = format!(
            r#"
name: Ack test
slug: ack-test
on: {{ type: presence, sensor: hallway, present: true }}
modes: [home]
run:
  - type: notify
    notify: {{ type: android_app }}
    category: general
    message: "Bins"
{step}
"#
        );

        Config::builder()
            .add_source(File::from_str(&yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Workflow>()
            .unwrap()
    }

    #[test]
    fn notify_step_parses_acknowledge() {
        let workflow = notify_workflow(
            "    acknowledge: { remind_after: 2h, reminders: 1 }\n    actions:\n      - { label: Done, action: { type: acknowledge } }",
        );

        let Step::Notify { acknowledge, .. } = &workflow.run[0] else {
            panic!("expected a notify step");
        };

        let acknowledge = acknowledge.expect("acknowledge block parsed");
        assert_eq!(acknowledge.remind_after, TimeDelta::hours(2));
        assert_eq!(acknowledge.reminders, 1);
        assert!(workflow.validate_acknowledgements().is_ok());
    }

    #[test]
    fn acknowledge_block_requires_an_acknowledge_action() {
        let workflow = notify_workflow("    acknowledge: { remind_after: 2h, reminders: 1 }");

        let err = workflow.validate_acknowledgements().unwrap_err();
        assert!(err.contains("no acknowledge action"), "{err}");
    }

    #[test]
    fn acknowledge_action_requires_an_acknowledge_block() {
        let workflow =
            notify_workflow("    actions:\n      - { label: Done, action: { type: acknowledge } }");

        let err = workflow.validate_acknowledgements().unwrap_err();
        assert!(err.contains("no `acknowledge` block"), "{err}");
    }

    #[test]
    fn acknowledge_block_requires_both_fields() {
        let yaml = r#"
name: Ack test
slug: ack-test
on: { type: presence, sensor: hallway, present: true }
modes: [home]
run:
  - type: notify
    notify: { type: android_app }
    category: general
    message: "Bins"
    acknowledge: { remind_after: 2h }
"#;

        let err = Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Workflow>()
            .unwrap_err();

        assert!(err.to_string().contains("reminders"), "{err}");
    }
}
