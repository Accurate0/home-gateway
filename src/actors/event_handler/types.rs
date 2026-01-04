use crate::{
    actors::{
        devices::{control_switch::ControlSwitchHandler, presence_sensor::PresenceSensorHandler},
        door_sensor::DoorSensorHandler,
        light::LightHandler,
        smart_switch::SmartSwitchHandler,
        temperature_sensor::TemperatureSensorHandler,
    },
    zigbee2mqtt::{
        Aqara_FP1E, Aqara_MCCGQ12LM, Aqara_T1, Aqara_WSDCGQ12LM, Aqara_WXKG11LM, IKEA_E2001,
        IKEA_E2112, IKEA_LED2201G8, Lumi_WSDCGQ11LM, Phillips_9290012573A, TS011F_plug_1,
    },
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum GenericZigbee2MqttMessage {
    TS011FSmartSwitch(TS011F_plug_1::Ts011fPlug1),
    AqaraPresenceSensor(Aqara_FP1E::AqaraFP1E),
    AqaraSingleButtonSwitch(Aqara_WXKG11LM::AqaraWXKG11LM),
    AqaraTemperatureSensor(Aqara_WSDCGQ12LM::AqaraWSDCGQ12LM),
    AqaraWhiteLight(Aqara_T1::AqaraT1),
    LumiTemperatureSensor(Lumi_WSDCGQ11LM::LumiWSDCGQ11LM),
    AquaraDoorSensor(Aqara_MCCGQ12LM::AqaraMCCGQ12LM),
    PhillipsLight(Phillips_9290012573A::Phillips9290012573A),
    IKEATemperatureSensor(IKEA_E2112::IKEAE2112),
    IKEALight(IKEA_LED2201G8::IKEALED2201G8),
    IKEASwitch(IKEA_E2001::IKEAE2001),
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
            GenericZigbee2MqttMessage::LumiTemperatureSensor(msg) => {
                write!(f, "Lumi Temperature Sensor: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::IKEASwitch(msg) => {
                write!(f, "IKEA Switch: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::IKEATemperatureSensor(msg) => {
                write!(f, "IKEA Temperature Sensor: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::AqaraPresenceSensor(msg) => {
                write!(f, "Aqara Presence Sensor: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::AqaraWhiteLight(msg) => {
                write!(f, "Aqara T1 Light: {}", msg.device.friendly_name)
            }
            GenericZigbee2MqttMessage::AqaraSingleButtonSwitch(msg) => {
                write!(
                    f,
                    "Aqara Single Button Switch: {}",
                    msg.device.friendly_name
                )
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
            GenericZigbee2MqttMessage::LumiTemperatureSensor(lumi_wsdcgq11_lm) => {
                &lumi_wsdcgq11_lm.device.ieee_addr
            }
            GenericZigbee2MqttMessage::IKEASwitch(ikeae2001) => &ikeae2001.device.ieee_addr,
            GenericZigbee2MqttMessage::IKEATemperatureSensor(ikeae2112) => {
                &ikeae2112.device.ieee_addr
            }
            GenericZigbee2MqttMessage::AqaraPresenceSensor(aqara_fp1_e) => {
                &aqara_fp1_e.device.ieee_addr
            }
            GenericZigbee2MqttMessage::AqaraWhiteLight(aqara_t1) => &aqara_t1.device.ieee_addr,
            GenericZigbee2MqttMessage::AqaraSingleButtonSwitch(aqara_wxkg11_lm) => {
                &aqara_wxkg11_lm.device.ieee_addr
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
            GenericZigbee2MqttMessage::LumiTemperatureSensor(lumi_wsdcgq11_lm) => {
                &lumi_wsdcgq11_lm.device.friendly_name
            }
            GenericZigbee2MqttMessage::IKEASwitch(ikeae2001) => &ikeae2001.device.friendly_name,
            GenericZigbee2MqttMessage::IKEATemperatureSensor(ikeae2112) => {
                &ikeae2112.device.friendly_name
            }
            GenericZigbee2MqttMessage::AqaraPresenceSensor(aqara_fp1_e) => {
                &aqara_fp1_e.device.friendly_name
            }
            GenericZigbee2MqttMessage::AqaraWhiteLight(aqara_t1) => &aqara_t1.device.friendly_name,
            GenericZigbee2MqttMessage::AqaraSingleButtonSwitch(aqara_wxkg11_lm) => {
                &aqara_wxkg11_lm.device.friendly_name
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
            GenericZigbee2MqttMessage::LumiTemperatureSensor(_) => {
                TypedActorName::TemperatureSensor
            }
            GenericZigbee2MqttMessage::IKEASwitch(_) => TypedActorName::ControlSwitch,
            GenericZigbee2MqttMessage::IKEATemperatureSensor(_) => {
                TypedActorName::TemperatureSensor
            }
            GenericZigbee2MqttMessage::AqaraPresenceSensor(_) => TypedActorName::PresenceSensor,
            GenericZigbee2MqttMessage::AqaraWhiteLight(_) => TypedActorName::Light,
            GenericZigbee2MqttMessage::AqaraSingleButtonSwitch(_) => TypedActorName::ControlSwitch,
        }
    }
}

pub enum TypedActorName {
    SmartSwitch,
    TemperatureSensor,
    PresenceSensor,
    DoorSensor,
    Light,
    ControlSwitch,
}

impl std::fmt::Display for TypedActorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedActorName::SmartSwitch => write!(f, "{}", SmartSwitchHandler::NAME),
            TypedActorName::TemperatureSensor => write!(f, "{}", TemperatureSensorHandler::NAME),
            TypedActorName::DoorSensor => write!(f, "{}", DoorSensorHandler::NAME),
            TypedActorName::Light => write!(f, "{}", LightHandler::NAME),
            TypedActorName::ControlSwitch => write!(f, "{}", ControlSwitchHandler::NAME),
            TypedActorName::PresenceSensor => write!(f, "{}", PresenceSensorHandler::NAME),
        }
    }
}
