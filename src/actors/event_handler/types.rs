use crate::{
    actors::{
        door_sensor::DoorSensorHandler, light::LightHandler, smart_switch::SmartSwitchHandler,
        temperature_sensor::TemperatureSensorHandler,
    },
    zigbee2mqtt::{
        Aqara_MCCGQ12LM, Aqara_WSDCGQ12LM, IKEA_LED2201G8, Phillips_9290012573A, TS011F_plug_1,
    },
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum GenericZigbee2MqttMessage {
    TS011FSmartSwitch(TS011F_plug_1::Ts011fPlug1),
    AqaraTemperatureSensor(Aqara_WSDCGQ12LM::AqaraWSDCGQ12LM),
    AquaraDoorSensor(Aqara_MCCGQ12LM::AqaraMCCGQ12LM),
    PhillipsLight(Phillips_9290012573A::Phillips9290012573A),
    IKEALight(IKEA_LED2201G8::IKEALED2201G8),
}

impl std::fmt::Display for GenericZigbee2MqttMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericZigbee2MqttMessage::TS011FSmartSwitch(msg) => {
                write!(f, "TS011F Smart Switch: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(msg) => {
                write!(f, "Aqara Temperature Sensor: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::AquaraDoorSensor(msg) => {
                write!(f, "Aqara Door Sensor: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::PhillipsLight(msg) => {
                write!(f, "Phillips Light: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::IKEALight(msg) => {
                write!(f, "IKEA Light: {}", msg.device.friendly_name)
            }
        }
    }
}

impl GenericZigbee2MqttMessage {
    #[allow(unused)]
    pub fn to_ieee_addr(&self) -> &str {
        match self {
            GenericZigbee2MqttMessage::TS011FSmartSwitch(ts011f_plug1) => {
                &ts011f_plug1.device.ieee_addr
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(aqara_wsdcgq12_lm) => {
                &aqara_wsdcgq12_lm.device.ieee_addr
            }
            GenericZigbee2MqttMessage::AquaraDoorSensor(aqara_mccgq12_lm) => {
                &aqara_mccgq12_lm.device.ieee_addr
            }
            GenericZigbee2MqttMessage::PhillipsLight(phillips9290012573_a) => {
                &phillips9290012573_a.device.ieee_addr
            }
            GenericZigbee2MqttMessage::IKEALight(ikealed2201_g8) => {
                &ikealed2201_g8.device.ieee_addr
            }
        }
    }

    #[allow(unused)]
    pub fn to_friendly_name(&self) -> &str {
        match self {
            GenericZigbee2MqttMessage::TS011FSmartSwitch(ts011f_plug1) => {
                &ts011f_plug1.device.friendly_name
            }
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(aqara_wsdcgq12_lm) => {
                &aqara_wsdcgq12_lm.device.friendly_name
            }
            GenericZigbee2MqttMessage::AquaraDoorSensor(aqara_mccgq12_lm) => {
                &aqara_mccgq12_lm.device.friendly_name
            }
            GenericZigbee2MqttMessage::PhillipsLight(phillips9290012573_a) => {
                &phillips9290012573_a.device.friendly_name
            }
            GenericZigbee2MqttMessage::IKEALight(ikealed2201_g8) => {
                &ikealed2201_g8.device.friendly_name
            }
        }
    }
    pub fn to_actor_name(&self) -> TypedActorName {
        match self {
            GenericZigbee2MqttMessage::TS011FSmartSwitch(_) => TypedActorName::SmartSwitch,
            GenericZigbee2MqttMessage::AqaraTemperatureSensor(_) => {
                TypedActorName::TemperatureSensor
            }
            GenericZigbee2MqttMessage::AquaraDoorSensor(_) => TypedActorName::DoorSensor,
            GenericZigbee2MqttMessage::PhillipsLight(_) => TypedActorName::Light,
            GenericZigbee2MqttMessage::IKEALight(_) => TypedActorName::Light,
        }
    }
}

pub enum TypedActorName {
    SmartSwitch,
    TemperatureSensor,
    DoorSensor,
    Light,
}

impl std::fmt::Display for TypedActorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedActorName::SmartSwitch => write!(f, "{}", SmartSwitchHandler::NAME),
            TypedActorName::TemperatureSensor => write!(f, "{}", TemperatureSensorHandler::NAME),
            TypedActorName::DoorSensor => write!(f, "{}", DoorSensorHandler::NAME),
            TypedActorName::Light => write!(f, "{}", LightHandler::NAME),
        }
    }
}
