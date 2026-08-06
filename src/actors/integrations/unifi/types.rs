use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiWebhookEvent {
    #[serde(rename = "alarm_id")]
    pub alarm_id: String,
    pub app: String,
    pub custom_content: Value,
    pub device_event_class_id: String,
    pub message: String,
    pub name: String,
    pub parameters: Parameters,
    pub severity: i64,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Parameters {
    Connect(ConnectParameters),
    Disconnect(DisconnectParameters),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisconnectParameters {
    #[serde(rename = "UNIFIcategory")]
    pub unificategory: Option<String>,
    #[serde(rename = "UNIFIclientAlias")]
    pub unificlient_alias: Option<String>,
    #[serde(rename = "UNIFIclientHostname")]
    pub unificlient_hostname: Option<String>,
    #[serde(rename = "UNIFIclientIp")]
    pub unificlient_ip: Option<String>,
    #[serde(rename = "UNIFIclientMac")]
    pub unificlient_mac: String,
    #[serde(rename = "UNIFIduration")]
    pub unifiduration: Option<String>,
    #[serde(rename = "UNIFIhost")]
    pub unifihost: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToDeviceIp")]
    pub unifilast_connected_to_device_ip: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToDeviceMac")]
    pub unifilast_connected_to_device_mac: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToDeviceModel")]
    pub unifilast_connected_to_device_model: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToDeviceName")]
    pub unifilast_connected_to_device_name: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToDeviceVersion")]
    pub unifilast_connected_to_device_version: Option<String>,
    #[serde(rename = "UNIFIlastConnectedToWiFiRssi")]
    pub unifilast_connected_to_wi_fi_rssi: Option<String>,
    #[serde(rename = "UNIFInetworkName")]
    pub unifinetwork_name: Option<String>,
    #[serde(rename = "UNIFInetworkSubnet")]
    pub unifinetwork_subnet: Option<String>,
    #[serde(rename = "UNIFInetworkVlan")]
    pub unifinetwork_vlan: Option<String>,
    #[serde(rename = "UNIFIusageDown")]
    pub unifiusage_down: Option<String>,
    #[serde(rename = "UNIFIusageUp")]
    pub unifiusage_up: Option<String>,
    #[serde(rename = "UNIFIutcTime")]
    pub unifiutc_time: Option<String>,
    #[serde(rename = "UNIFIwifiAirtimeUtilization")]
    pub unifiwifi_airtime_utilization: Option<String>,
    #[serde(rename = "UNIFIwifiBand")]
    pub unifiwifi_band: Option<String>,
    #[serde(rename = "UNIFIwifiChannel")]
    pub unifiwifi_channel: Option<String>,
    #[serde(rename = "UNIFIwifiChannelWidth")]
    pub unifiwifi_channel_width: Option<String>,
    #[serde(rename = "UNIFIwifiInterference")]
    pub unifiwifi_interference: Option<String>,
    #[serde(rename = "UNIFIwifiName")]
    pub unifiwifi_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParameters {
    #[serde(rename = "UNIFIWiFiRssi")]
    pub unifiwi_fi_rssi: Option<String>,
    #[serde(rename = "UNIFIauthMethod")]
    pub unifiauth_method: Option<String>,
    #[serde(rename = "UNIFIcategory")]
    pub unificategory: Option<String>,
    #[serde(rename = "UNIFIclientAlias")]
    pub unificlient_alias: Option<String>,
    #[serde(rename = "UNIFIclientHostname")]
    pub unificlient_hostname: Option<String>,
    #[serde(rename = "UNIFIclientIp")]
    pub unificlient_ip: Option<String>,
    #[serde(rename = "UNIFIclientMac")]
    pub unificlient_mac: String,
    #[serde(rename = "UNIFIconnectedToDeviceIp")]
    pub unificonnected_to_device_ip: Option<String>,
    #[serde(rename = "UNIFIconnectedToDeviceMac")]
    pub unificonnected_to_device_mac: Option<String>,
    #[serde(rename = "UNIFIconnectedToDeviceModel")]
    pub unificonnected_to_device_model: Option<String>,
    #[serde(rename = "UNIFIconnectedToDeviceName")]
    pub unificonnected_to_device_name: Option<String>,
    #[serde(rename = "UNIFIconnectedToDeviceVersion")]
    pub unificonnected_to_device_version: Option<String>,
    #[serde(rename = "UNIFIhost")]
    pub unifihost: Option<String>,
    #[serde(rename = "UNIFInetworkName")]
    pub unifinetwork_name: Option<String>,
    #[serde(rename = "UNIFInetworkSubnet")]
    pub unifinetwork_subnet: Option<String>,
    #[serde(rename = "UNIFInetworkVlan")]
    pub unifinetwork_vlan: Option<String>,
    #[serde(rename = "UNIFIutcTime")]
    pub unifiutc_time: Option<String>,
    #[serde(rename = "UNIFIwifiBand")]
    pub unifiwifi_band: Option<String>,
    #[serde(rename = "UNIFIwifiChannel")]
    pub unifiwifi_channel: Option<String>,
    #[serde(rename = "UNIFIwifiChannelWidth")]
    pub unifiwifi_channel_width: Option<String>,
    #[serde(rename = "UNIFIwifiName")]
    pub unifiwifi_name: Option<String>,
}
