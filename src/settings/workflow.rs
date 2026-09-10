use crate::device_registry::{Capability, DeviceRegistry};
use crate::event_bus::{ForecastDay, SolarMetric, WeatherMetric, WeatherSource};
use crate::settings::TemplateString;
use crate::settings::http_method::HttpMethod;
use crate::settings::switch_metric::SwitchMetric;
use crate::settings::trigger::TriggerMatcher;
use crate::settings::vacuum_command::VacuumCommand;
use crate::settings::workflow_timers::WorkflowTimerSettings;
use crate::settings::{NotifyAcknowledge, NotifyAction, NotifyCategory, NotifySource};
use crate::timedelta_format::option_time_delta_from_str;
use std::collections::BTreeMap;

use super::{DeviceAliases, IEEEAddress, ReusableWorkflow, validate_device};
use crate::actors::sun::calc::SunPeriod;
use crate::mode::Mode;
use chrono::{NaiveTime, TimeDelta};
use schemars::JsonSchema;
use serde::Deserialize;

/// Brightness / colour-temperature mutations applied to a light. Kept in
/// `SCREAMING_SNAKE_CASE` to match the long-standing on-disk config.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LightState {
    On,
    Off,
    Toggle,
    SetBrightness {
        value: u64,
    },
    IncreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    DecreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    IncreaseColourTemperature {
        value: u64,
    },
    DecreaseColourTemperature {
        value: u64,
    },
    StopColourTemperature,
    StopBrightness,
}

impl LightState {
    pub fn required_capability(&self) -> Option<Capability> {
        match self {
            LightState::On | LightState::Off | LightState::Toggle => None,
            LightState::SetBrightness { .. }
            | LightState::IncreaseBrightness { .. }
            | LightState::DecreaseBrightness { .. }
            | LightState::StopBrightness => Some(Capability::Brightness),
            LightState::IncreaseColourTemperature { .. }
            | LightState::DecreaseColourTemperature { .. }
            | LightState::StopColourTemperature => Some(Capability::ColourTemp),
        }
    }
}

/// Target enablement for a `set_workflows_enabled` step. `Toggle` flips the
/// whole tagged set together, based on whether any member is currently enabled.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnableState {
    Enabled,
    Disabled,
    Toggle,
}

/// On/off set command for a smart switch / plug.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwitchState {
    On,
    Off,
    Toggle,
}

/// Which reading of an environment sensor a condition compares against.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvMetric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
}

#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
}

/// A scalar comparison: `{ op: gt, value: 30 }`. Flattened into the
/// [`Condition::Environment`] variant.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
pub struct Comparison {
    pub op: CompareOp,
    pub value: f64,
}

impl Comparison {
    pub fn matches(&self, actual: f64) -> bool {
        match self.op {
            CompareOp::Gt => actual > self.value,
            CompareOp::Lt => actual < self.value,
            CompareOp::Gte => actual >= self.value,
            CompareOp::Lte => actual <= self.value,
            // direct float equality is intentional: thresholds are configured as
            // exact values and sensors report discrete readings
            CompareOp::Eq => actual == self.value,
        }
    }
}

/// A boolean predicate evaluated against current device/sensor state. Either a
/// nested boolean combinator (`all`/`and`, `any`/`or`, `not`) or a leaf test.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Condition {
    Combinator(Combinator),
    Leaf(LeafCondition),
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub enum Combinator {
    #[serde(rename = "all", alias = "and")]
    All(Vec<Condition>),
    #[serde(rename = "any", alias = "or")]
    Any(Vec<Condition>),
    #[serde(rename = "not")]
    Not(Box<Condition>),
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LeafCondition {
    Light {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        on: bool,
    },
    Environment {
        sensor: String,
        metric: EnvMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    Presence {
        sensor: String,
        present: bool,
    },
    Door {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        open: bool,
    },
    TimeOfDay {
        #[serde(default)]
        after: Option<NaiveTime>,
        #[serde(default)]
        before: Option<NaiveTime>,
    },
    Sun {
        is: SunPeriod,
        #[serde(
            default,
            deserialize_with = "crate::timedelta_format::signed_time_delta_from_str::deserialize"
        )]
        #[schemars(with = "String")]
        offset: TimeDelta,
    },
    Mode {
        is: Mode,
    },
    Solar {
        metric: SolarMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    HomeAssistant {
        entity_id: String,
        state: String,
    },
    SmartSwitch {
        #[serde(rename = "device")]
        ieee_addr: IEEEAddress,
        metric: SwitchMetric,
        #[serde(flatten)]
        cmp: Comparison,
    },
    Weather {
        source: WeatherSource,
        metric: WeatherMetric,
        #[serde(default)]
        day: Option<ForecastDay>,
        #[serde(flatten)]
        cmp: Comparison,
    },
}

impl Condition {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Condition::Combinator(c) => c.resolve_devices(devices),
            Condition::Leaf(l) => l.resolve_devices(devices),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Condition::Combinator(c) => c.describe(),
            Condition::Leaf(l) => l.describe(),
        }
    }
}

impl Combinator {
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Combinator::All(conditions) | Combinator::Any(conditions) => {
                for c in conditions {
                    c.resolve_devices(devices)?;
                }
            }
            Combinator::Not(condition) => condition.resolve_devices(devices)?,
        }
        Ok(())
    }

    fn describe(&self) -> String {
        match self {
            Combinator::All(conditions) => format!("all[{}]", describe_join(conditions)),
            Combinator::Any(conditions) => format!("any[{}]", describe_join(conditions)),
            Combinator::Not(condition) => format!("not({})", condition.describe()),
        }
    }
}

impl LeafCondition {
    fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            LeafCondition::Light { ieee_addr, .. }
            | LeafCondition::Door { ieee_addr, .. }
            | LeafCondition::SmartSwitch { ieee_addr, .. } => {
                validate_device(ieee_addr, devices)?;
            }
            LeafCondition::Environment { .. }
            | LeafCondition::Presence { .. }
            | LeafCondition::TimeOfDay { .. }
            | LeafCondition::Mode { .. }
            | LeafCondition::Sun { .. }
            | LeafCondition::Solar { .. }
            | LeafCondition::HomeAssistant { .. } => {}
            LeafCondition::Weather {
                source,
                metric,
                day,
                ..
            } => metric.validate(*source, *day)?,
        }
        Ok(())
    }

    fn describe(&self) -> String {
        match self {
            LeafCondition::Light { ieee_addr, on } => {
                format!("light({ieee_addr}) is {}", if *on { "on" } else { "off" })
            }
            LeafCondition::Environment {
                sensor,
                metric,
                cmp,
            } => format!("env({sensor}).{metric:?} {:?} {}", cmp.op, cmp.value),
            LeafCondition::Presence { sensor, present } => {
                format!("presence({sensor}) is {present}")
            }
            LeafCondition::Door { ieee_addr, open } => {
                format!(
                    "door({ieee_addr}) is {}",
                    if *open { "open" } else { "closed" }
                )
            }
            LeafCondition::TimeOfDay { after, before } => match (after, before) {
                (Some(a), Some(b)) => format!("time in [{a}, {b})"),
                (Some(a), None) => format!("time after {a}"),
                (None, Some(b)) => format!("time before {b}"),
                (None, None) => "time always".to_string(),
            },
            LeafCondition::Sun { is, offset } => {
                if offset.is_zero() {
                    format!("sun is {is:?}")
                } else {
                    format!(
                        "sun is {is:?} (offset {})",
                        crate::timedelta_format::humanize(*offset)
                    )
                }
            }
            LeafCondition::Mode { is } => format!("mode is {}", is.as_str()),
            LeafCondition::Solar { metric, cmp } => {
                format!("solar.{} {:?} {}", metric.var_name(), cmp.op, cmp.value)
            }
            LeafCondition::HomeAssistant { entity_id, state } => {
                format!("ha({entity_id}) is {state}")
            }
            LeafCondition::SmartSwitch {
                ieee_addr,
                metric,
                cmp,
            } => format!(
                "switch({ieee_addr}).{} {:?} {}",
                metric.as_str(),
                cmp.op,
                cmp.value
            ),
            LeafCondition::Weather {
                source,
                metric,
                day,
                cmp,
            } => format!(
                "weather({}).{} {:?} {}",
                source.as_str(),
                metric.var_name(*day),
                cmp.op,
                cmp.value
            ),
        }
    }
}

fn describe_join(conditions: &[Condition]) -> String {
    conditions
        .iter()
        .map(Condition::describe)
        .collect::<Vec<_>>()
        .join(", ")
}

/// A single workflow step: one action, optionally guarded by a `when` condition.
/// Nesting (the old `conditional` block) is expressed as a guarded `scene`.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Light {
        // accepts a device alias (resolved at load time) or a raw address;
        // `ieeeAddr` kept as an alias for backwards compatibility with the HTTP execute route
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
        state: LightState,
        #[serde(default)]
        when: Option<Condition>,
    },
    Switch {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        state: SwitchState,
        #[serde(default)]
        when: Option<Condition>,
    },
    Scene {
        run: Vec<Step>,
        #[serde(default)]
        when: Option<Condition>,
    },
    Notify {
        notify: NotifySource,
        message: TemplateString,
        #[serde(default)]
        title: Option<TemplateString>,
        category: NotifyCategory,
        #[serde(default)]
        actions: Vec<NotifyAction>,
        #[serde(default)]
        acknowledge: Option<NotifyAcknowledge>,
        #[serde(default)]
        when: Option<Condition>,
    },
    Delay {
        seconds: u64,
        #[serde(default)]
        when: Option<Condition>,
    },
    RunWorkflow {
        workflow: String,
        #[serde(default)]
        when: Option<Condition>,
    },
    SetMode {
        mode: Mode,
        #[serde(default)]
        when: Option<Condition>,
    },
    SetWorkflowsEnabled {
        tag: String,
        state: EnableState,
        #[serde(default)]
        when: Option<Condition>,
    },
    HomeAssistant {
        #[serde(rename = "call_service")]
        call_service: String,
        #[serde(default)]
        data: serde_json::Value,
        #[serde(default)]
        when: Option<Condition>,
    },
    MqttPublish {
        topic: TemplateString,
        payload: TemplateString,
        retain: bool,
        #[serde(default)]
        when: Option<Condition>,
    },
    Http {
        method: HttpMethod,
        url: TemplateString,
        #[serde(default)]
        headers: BTreeMap<String, TemplateString>,
        #[serde(default)]
        body: Option<TemplateString>,
        #[serde(default)]
        when: Option<Condition>,
    },
    RobotVacuum {
        #[serde(rename = "device")]
        ieee_addr: IEEEAddress,
        command: VacuumCommand,
        #[serde(default)]
        when: Option<Condition>,
    },
}

impl Step {
    /// Static step kind, used as a label in logs, spans, and metrics.
    pub fn kind(&self) -> &'static str {
        match self {
            Step::Light { .. } => "light",
            Step::Switch { .. } => "switch",
            Step::Scene { .. } => "scene",
            Step::Notify { .. } => "notify",
            Step::Delay { .. } => "delay",
            Step::RunWorkflow { .. } => "run_workflow",
            Step::SetMode { .. } => "set_mode",
            Step::SetWorkflowsEnabled { .. } => "set_workflows_enabled",
            Step::HomeAssistant { .. } => "home_assistant",
            Step::MqttPublish { .. } => "mqtt_publish",
            Step::Http { .. } => "http",
            Step::RobotVacuum { .. } => "robot_vacuum",
        }
    }

    /// The optional guard condition shared across every step variant.
    pub fn guard(&self) -> Option<&Condition> {
        match self {
            Step::Light { when, .. }
            | Step::Switch { when, .. }
            | Step::Scene { when, .. }
            | Step::Notify { when, .. }
            | Step::Delay { when, .. }
            | Step::RunWorkflow { when, .. }
            | Step::SetMode { when, .. }
            | Step::SetWorkflowsEnabled { when, .. }
            | Step::HomeAssistant { when, .. }
            | Step::MqttPublish { when, .. }
            | Step::Http { when, .. }
            | Step::RobotVacuum { when, .. } => when.as_ref(),
        }
    }

    pub fn describe_action(&self) -> Option<String> {
        match self {
            Step::Light {
                ieee_addr, state, ..
            } => Some(format!("light({ieee_addr}) -> {state:?}")),
            Step::Switch {
                ieee_addr, state, ..
            } => Some(format!("switch({ieee_addr}) -> {state:?}")),
            Step::Notify {
                notify,
                message,
                category,
                actions,
                acknowledge,
                ..
            } => {
                let suffix = if actions.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<&str> = actions.iter().map(|a| a.label.as_str()).collect();
                    format!(" [{}]", labels.join(", "))
                };

                let ack = match acknowledge {
                    Some(ack) => format!(
                        " (remind after {}, {} reminder(s) until acknowledged)",
                        crate::timedelta_format::humanize(ack.remind_after),
                        ack.reminders
                    ),
                    None => String::new(),
                };

                Some(format!(
                    "notify({notify:?}, {}): {message}{suffix}{ack}",
                    category.as_str()
                ))
            }
            Step::Delay { seconds, .. } => Some(format!("delay {seconds}s")),
            Step::SetMode { mode, .. } => Some(format!("set_mode({})", mode.as_str())),
            Step::SetWorkflowsEnabled { tag, state, .. } => {
                Some(format!("set_workflows_enabled(#{tag}) -> {state:?}"))
            }
            Step::HomeAssistant {
                call_service, data, ..
            } => Some(format!("home_assistant({call_service}) {data}")),
            Step::MqttPublish {
                topic,
                payload,
                retain,
                ..
            } => {
                let suffix = if *retain { " (retained)" } else { "" };

                Some(format!("mqtt_publish({topic}) {payload}{suffix}"))
            }
            Step::Http { method, url, .. } => Some(format!("http {method:?} {url}")),
            Step::RobotVacuum {
                ieee_addr, command, ..
            } => Some(format!("robot_vacuum({ieee_addr}) -> {command:?}")),
            Step::Scene { .. } | Step::RunWorkflow { .. } => None,
        }
    }

    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Step::Light {
                ieee_addr, when, ..
            }
            | Step::Switch {
                ieee_addr, when, ..
            }
            | Step::RobotVacuum {
                ieee_addr, when, ..
            } => {
                validate_device(ieee_addr, devices)?;
                resolve_opt(when, devices)?;
            }
            Step::Scene { run, when } => {
                for step in run {
                    step.resolve_devices(devices)?;
                }
                resolve_opt(when, devices)?;
            }
            Step::Notify { when, .. }
            | Step::Delay { when, .. }
            | Step::RunWorkflow { when, .. }
            | Step::SetMode { when, .. }
            | Step::SetWorkflowsEnabled { when, .. }
            | Step::HomeAssistant { when, .. }
            | Step::MqttPublish { when, .. }
            | Step::Http { when, .. } => resolve_opt(when, devices)?,
        }
        Ok(())
    }

    pub(super) fn validate_capabilities(&self, registry: &DeviceRegistry) -> Result<(), String> {
        match self {
            Step::Light {
                ieee_addr, state, ..
            } => {
                let address = registry.address_or_self(ieee_addr);
                if let Some(required) = state.required_capability()
                    && !registry.capabilities(address).contains(&required)
                {
                    return Err(format!(
                        "light {ieee_addr} does not support {required:?}: {state:?}"
                    ));
                }
            }
            Step::Switch { ieee_addr, .. } => {
                let address = registry.address_or_self(ieee_addr);
                if registry.light(address).is_none() {
                    return Err(format!(
                        "switch {ieee_addr} cannot be driven: only a smart switch declared \
                         `as: light` has a control path"
                    ));
                }
            }
            Step::RobotVacuum { ieee_addr, .. } => {
                let address = registry.address_or_self(ieee_addr);
                if registry.roborock(address).is_none() && registry.valetudo(address).is_none() {
                    return Err(format!("robot_vacuum {ieee_addr} is not a robot vacuum"));
                }
            }
            Step::Scene { run, .. } => {
                for step in run {
                    step.validate_capabilities(registry)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn resolve_opt(when: &mut Option<Condition>, devices: &DeviceAliases) -> Result<(), String> {
    if let Some(when) = when {
        when.resolve_devices(devices)?;
    }
    Ok(())
}

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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WorkflowSettings {
    pub workers: usize,
    pub timers: WorkflowTimerSettings,
}

impl Workflow {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        self.on.resolve_devices(devices)?;
        resolve_opt(&mut self.when, devices)?;

        self.body.resolve_devices(devices)
    }
}

#[cfg(test)]
mod fuelwatch_trigger_tests {
    use super::*;

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
        assert!(on.available_vars().iter().any(|var| var == "new_price"));
        assert_eq!(on.describe(), "fuelwatch(*) tomorrow_lower drop >= 5");
    }
}

#[cfg(test)]
mod outbound_step_tests {
    use super::*;

    #[test]
    fn mqtt_publish_http_and_robot_vacuum_steps_parse() {
        let steps: Vec<Step> = serde_yaml::from_str(
            r#"
- type: mqtt_publish
  topic: "home/${room}/scene"
  payload: '{"scene": "movie"}'
  retain: true
- type: http
  method: POST
  url: "https://example.com/hook/${name}"
  headers: { Authorization: "Bearer x" }
  body: '{"event": "${name}"}'
- type: robot_vacuum
  device: roborock
  command: dock
"#,
        )
        .unwrap();

        assert_eq!(
            steps.iter().map(Step::kind).collect::<Vec<_>>(),
            ["mqtt_publish", "http", "robot_vacuum"]
        );

        match &steps[0] {
            Step::MqttPublish { topic, retain, .. } => {
                assert_eq!(topic.placeholders(), vec!["room"]);
                assert!(*retain);
            }
            other => panic!("expected mqtt_publish, got {}", other.kind()),
        }

        match &steps[1] {
            Step::Http {
                method,
                headers,
                body,
                ..
            } => {
                assert_eq!(*method, HttpMethod::Post);
                assert_eq!(headers.len(), 1);
                assert!(body.is_some());
            }
            other => panic!("expected http, got {}", other.kind()),
        }

        match &steps[2] {
            Step::RobotVacuum {
                ieee_addr, command, ..
            } => {
                assert_eq!(ieee_addr, "roborock");
                assert_eq!(*command, VacuumCommand::Dock);
            }
            other => panic!("expected robot_vacuum, got {}", other.kind()),
        }
    }

    #[test]
    fn mqtt_publish_requires_retain() {
        let parsed = serde_yaml::from_str::<Step>("type: mqtt_publish\ntopic: a\npayload: b\n");

        assert!(parsed.is_err());
    }
}

#[cfg(test)]
mod condition_tests {
    use super::*;
    use config::{Config, File, FileFormat};

    #[derive(serde::Deserialize)]
    struct Wrap {
        when: Condition,
    }

    fn parse(yaml: &str) -> Condition {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Wrap>()
            .unwrap()
            .when
    }

    #[test]
    fn nested_or_and_not_map_keys() {
        let cond = parse(
            r#"
when:
  or:
    - type: mode
      is: guest
    - and:
        - type: presence
          sensor: living-room
          present: true
        - not:
            type: mode
            is: vacation
"#,
        );
        assert_eq!(
            cond.describe(),
            "any[mode is guest, all[presence(living-room) is true, not(mode is vacation)]]"
        );
    }

    #[test]
    fn solar_home_assistant_and_smart_switch_leaves_parse() {
        let cond = parse(
            r#"
when:
  all:
    - type: solar
      metric: avg_15m
      op: gt
      value: 3000
    - type: home_assistant
      entity_id: binary_sensor.washer
      state: "off"
    - type: smart_switch
      device: "0x1"
      metric: power
      op: lt
      value: 5
"#,
        );
        assert_eq!(
            cond.describe(),
            "all[solar.avg_15m Gt 3000, ha(binary_sensor.washer) is off, switch(0x1).power Lt 5]"
        );
    }

    #[test]
    fn weather_leaf_parses_with_day() {
        let cond = parse(
            r#"
when:
  type: weather
  source: willyweather
  metric: max_temp
  day: tomorrow
  op: gte
  value: 35
"#,
        );
        assert_eq!(
            cond.describe(),
            "weather(willyweather).tomorrow_max_temp Gte 35"
        );
    }

    #[test]
    fn all_any_aliases_match_and_or() {
        let with_all = parse("when:\n  all:\n    - type: mode\n      is: guest\n");
        let with_and = parse("when:\n  and:\n    - type: mode\n      is: guest\n");
        assert_eq!(with_all.describe(), with_and.describe());
    }
}

#[cfg(test)]
mod home_assistant_tests {
    use super::*;
    use crate::settings::NotifyActionKind;
    use crate::settings::trigger::TriggerMatcher;
    use crate::settings::workflow_context::ContextSource;
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
    message: "${fuel_price}c/L at ${fuel_brand}"
"#,
        );

        assert_eq!(workflow.context, vec![ContextSource::Fuelwatch]);

        for var in workflow.template_placeholders() {
            assert!(
                ContextSource::Fuelwatch
                    .available_vars()
                    .contains(&var.as_str())
            );
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
