use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use super::condition::resolve_opt;
use super::{Condition, EnableState, HttpMethod, LightState, SwitchState, VacuumCommand};
use crate::device_registry::DeviceRegistry;
use crate::mode::Mode;
use crate::settings::{
    DeviceAliases, IEEEAddress, NotifyAcknowledge, NotifyAction, NotifyCategory, NotifySource,
    validate_device,
};
use crate::templating::Template;

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
        message: Template,
        #[serde(default)]
        title: Option<Template>,
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
        with: BTreeMap<String, Template>,
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
        topic: Template,
        payload: Template,
        retain: bool,
        #[serde(default)]
        when: Option<Condition>,
    },
    Http {
        method: HttpMethod,
        url: Template,
        #[serde(default)]
        headers: BTreeMap<String, Template>,
        #[serde(default)]
        body: Option<Template>,
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

    pub fn templates(&self) -> Vec<(String, &Template)> {
        match self {
            Step::Notify { message, title, .. } => std::iter::once(("message".to_owned(), message))
                .chain(title.iter().map(|title| ("title".to_owned(), title)))
                .collect(),
            Step::MqttPublish { topic, payload, .. } => {
                vec![("topic".to_owned(), topic), ("payload".to_owned(), payload)]
            }
            Step::Http {
                url, headers, body, ..
            } => std::iter::once(("url".to_owned(), url))
                .chain(
                    headers
                        .iter()
                        .map(|(name, value)| (format!("headers.{name}"), value)),
                )
                .chain(body.iter().map(|body| ("body".to_owned(), body)))
                .collect(),
            Step::RunWorkflow { with, .. } => with
                .iter()
                .map(|(key, value)| (format!("with.{key}"), value))
                .collect(),
            Step::Light { .. }
            | Step::Switch { .. }
            | Step::Scene { .. }
            | Step::Delay { .. }
            | Step::SetMode { .. }
            | Step::SetWorkflowsEnabled { .. }
            | Step::HomeAssistant { .. }
            | Step::RobotVacuum { .. } => Vec::new(),
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

    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
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

    pub(crate) fn validate_capabilities(&self, registry: &DeviceRegistry) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_publish_http_and_robot_vacuum_steps_parse() {
        let steps: Vec<Step> = serde_yaml::from_str(
            r#"
- type: mqtt_publish
  topic: "home/${event.room}/scene"
  payload: '{"scene": "movie"}'
  retain: true
- type: http
  method: POST
  url: "https://example.com/hook/${event.name}"
  headers: { Authorization: "Bearer x" }
  body: '{"event": "${event.name}"}'
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
                assert_eq!(
                    topic
                        .exprs()
                        .map(|expr| expr.path.to_string())
                        .collect::<Vec<_>>(),
                    vec!["event.room"]
                );
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
