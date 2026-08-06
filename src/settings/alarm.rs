use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

fn default_alarm_offset() -> TimeDelta {
    TimeDelta::minutes(5)
}

fn default_alarm_workflow() -> String {
    "alarm-wakeup".to_owned()
}

fn default_alarm_poll_interval() -> TimeDelta {
    TimeDelta::seconds(60)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AlarmSettings {
    #[serde(default = "default_alarm_offset", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub offset: TimeDelta,
    #[serde(default = "default_alarm_workflow")]
    pub workflow: String,
    #[serde(default = "default_alarm_poll_interval", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub poll_interval: TimeDelta,
}

impl Default for AlarmSettings {
    fn default() -> Self {
        Self {
            offset: default_alarm_offset(),
            workflow: default_alarm_workflow(),
            poll_interval: default_alarm_poll_interval(),
        }
    }
}
