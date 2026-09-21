use crate::integrations::mqtt::{MqttProtocol, TopicVars};
use crate::settings::MqttProtocols;

pub struct TopicMatch {
    pub protocol: MqttProtocol,
    pub stream: String,
    pub vars: TopicVars,
}

pub enum MqttTopic {
    Directory(MqttProtocol),
    Discovery {
        protocol: MqttProtocol,
        address: String,
    },
    Report(Vec<TopicMatch>),
    Unhandled,
}

impl MqttTopic {
    pub fn classify(protocols: &MqttProtocols, topic: &str) -> Self {
        for (protocol, settings) in protocols.iter() {
            if settings
                .directory
                .as_ref()
                .is_some_and(|directory| directory.matches(topic).is_some())
            {
                return MqttTopic::Directory(protocol);
            }

            let discovered = settings
                .discovery
                .as_ref()
                .and_then(|discovery| discovery.matches(topic))
                .and_then(|mut vars| vars.remove("address"));

            if let Some(address) = discovered {
                return MqttTopic::Discovery { protocol, address };
            }
        }

        let matches: Vec<TopicMatch> = protocols
            .iter()
            .flat_map(|(protocol, settings)| {
                settings
                    .topics
                    .iter()
                    .filter_map(move |(stream, template)| {
                        Some(TopicMatch {
                            protocol,
                            stream: stream.clone(),
                            vars: template.matches(topic)?,
                        })
                    })
            })
            .collect();

        if matches.is_empty() {
            MqttTopic::Unhandled
        } else {
            MqttTopic::Report(matches)
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            MqttTopic::Directory(_) => "directory",
            MqttTopic::Discovery { .. } => "discovery",
            MqttTopic::Report(_) => "report",
            MqttTopic::Unhandled => "unhandled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protocols() -> MqttProtocols {
        let section: serde_yaml::Value =
            serde_yaml::from_str(include_str!("../../../../config/sections/mqtt.yaml")).unwrap();

        serde_yaml::from_value(section["protocols"].clone()).unwrap()
    }

    fn report(topic: &str) -> Vec<(MqttProtocol, String, TopicVars)> {
        match MqttTopic::classify(&protocols(), topic) {
            MqttTopic::Report(matches) => matches
                .into_iter()
                .map(|found| (found.protocol, found.stream, found.vars))
                .collect(),
            MqttTopic::Directory(_) | MqttTopic::Discovery { .. } | MqttTopic::Unhandled => {
                panic!("expected {topic} to be a report")
            }
        }
    }

    fn vars(pairs: &[(&str, &str)]) -> TopicVars {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn control_topics_are_routed_before_reports() {
        assert!(matches!(
            MqttTopic::classify(&protocols(), "zigbee2mqtt/bridge/devices"),
            MqttTopic::Directory(MqttProtocol::Zigbee)
        ));
        assert!(matches!(
            MqttTopic::classify(&protocols(), "esphome/discover/apollo-mtr-1-livingroom"),
            MqttTopic::Discovery { protocol: MqttProtocol::Esphome, address }
                if address == "apollo-mtr-1-livingroom"
        ));
    }

    #[test]
    fn reports_carry_their_protocol_stream_and_vars() {
        assert_eq!(
            report("zigbee2mqtt/0x00158d008bbe0316"),
            [(
                MqttProtocol::Zigbee,
                "report".to_owned(),
                vars(&[("name", "0x00158d008bbe0316")])
            )]
        );
        assert_eq!(
            report("apollo-mtr-1-livingroom/binary_sensor/ld2450_moving_target/state"),
            [(
                MqttProtocol::Esphome,
                "state".to_owned(),
                vars(&[
                    ("address", "apollo-mtr-1-livingroom"),
                    ("domain", "binary_sensor"),
                    ("object_id", "ld2450_moving_target"),
                ])
            )]
        );
        assert_eq!(
            report("valetudo/rockrobo/attributes"),
            [(
                MqttProtocol::Valetudo,
                "attributes".to_owned(),
                vars(&[("address", "rockrobo")])
            )]
        );
    }

    #[test]
    fn topics_no_protocol_claims_are_unhandled() {
        assert!(matches!(
            MqttTopic::classify(&protocols(), "valetudo/rockrobo/map_data"),
            MqttTopic::Unhandled
        ));
    }
}
