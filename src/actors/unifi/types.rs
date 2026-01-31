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
    pub unificategory: String,
    #[serde(rename = "UNIFIclientAlias")]
    pub unificlient_alias: String,
    #[serde(rename = "UNIFIclientHostname")]
    pub unificlient_hostname: String,
    #[serde(rename = "UNIFIclientIp")]
    pub unificlient_ip: String,
    #[serde(rename = "UNIFIclientMac")]
    pub unificlient_mac: String,
    #[serde(rename = "UNIFIduration")]
    pub unifiduration: String,
    #[serde(rename = "UNIFIhost")]
    pub unifihost: String,
    #[serde(rename = "UNIFIlastConnectedToDeviceIp")]
    pub unifilast_connected_to_device_ip: String,
    #[serde(rename = "UNIFIlastConnectedToDeviceMac")]
    pub unifilast_connected_to_device_mac: String,
    #[serde(rename = "UNIFIlastConnectedToDeviceModel")]
    pub unifilast_connected_to_device_model: String,
    #[serde(rename = "UNIFIlastConnectedToDeviceName")]
    pub unifilast_connected_to_device_name: String,
    #[serde(rename = "UNIFIlastConnectedToDeviceVersion")]
    pub unifilast_connected_to_device_version: String,
    #[serde(rename = "UNIFIlastConnectedToWiFiRssi")]
    pub unifilast_connected_to_wi_fi_rssi: String,
    #[serde(rename = "UNIFInetworkName")]
    pub unifinetwork_name: String,
    #[serde(rename = "UNIFInetworkSubnet")]
    pub unifinetwork_subnet: String,
    #[serde(rename = "UNIFInetworkVlan")]
    pub unifinetwork_vlan: String,
    #[serde(rename = "UNIFIsubCategory")]
    pub unifisub_category: String,
    #[serde(rename = "UNIFIusageDown")]
    pub unifiusage_down: String,
    #[serde(rename = "UNIFIusageUp")]
    pub unifiusage_up: String,
    #[serde(rename = "UNIFIutcTime")]
    pub unifiutc_time: String,
    #[serde(rename = "UNIFIwifiAirtimeUtilization")]
    pub unifiwifi_airtime_utilization: String,
    #[serde(rename = "UNIFIwifiBand")]
    pub unifiwifi_band: String,
    #[serde(rename = "UNIFIwifiChannel")]
    pub unifiwifi_channel: String,
    #[serde(rename = "UNIFIwifiChannelWidth")]
    pub unifiwifi_channel_width: String,
    #[serde(rename = "UNIFIwifiInterference")]
    pub unifiwifi_interference: String,
    #[serde(rename = "UNIFIwifiName")]
    pub unifiwifi_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParameters {
    #[serde(rename = "UNIFIWiFiRssi")]
    pub unifiwi_fi_rssi: String,
    #[serde(rename = "UNIFIauthMethod")]
    pub unifiauth_method: String,
    #[serde(rename = "UNIFIcategory")]
    pub unificategory: String,
    #[serde(rename = "UNIFIclientAlias")]
    pub unificlient_alias: String,
    #[serde(rename = "UNIFIclientHostname")]
    pub unificlient_hostname: String,
    #[serde(rename = "UNIFIclientIp")]
    pub unificlient_ip: String,
    #[serde(rename = "UNIFIclientMac")]
    pub unificlient_mac: String,
    #[serde(rename = "UNIFIconnectedToDeviceIp")]
    pub unificonnected_to_device_ip: String,
    #[serde(rename = "UNIFIconnectedToDeviceMac")]
    pub unificonnected_to_device_mac: String,
    #[serde(rename = "UNIFIconnectedToDeviceModel")]
    pub unificonnected_to_device_model: String,
    #[serde(rename = "UNIFIconnectedToDeviceName")]
    pub unificonnected_to_device_name: String,
    #[serde(rename = "UNIFIconnectedToDeviceVersion")]
    pub unificonnected_to_device_version: String,
    #[serde(rename = "UNIFIhost")]
    pub unifihost: String,
    #[serde(rename = "UNIFInetworkName")]
    pub unifinetwork_name: String,
    #[serde(rename = "UNIFInetworkSubnet")]
    pub unifinetwork_subnet: String,
    #[serde(rename = "UNIFInetworkVlan")]
    pub unifinetwork_vlan: String,
    #[serde(rename = "UNIFIsubCategory")]
    pub unifisub_category: String,
    #[serde(rename = "UNIFIutcTime")]
    pub unifiutc_time: String,
    #[serde(rename = "UNIFIwifiBand")]
    pub unifiwifi_band: String,
    #[serde(rename = "UNIFIwifiChannel")]
    pub unifiwifi_channel: String,
    #[serde(rename = "UNIFIwifiChannelWidth")]
    pub unifiwifi_channel_width: String,
    #[serde(rename = "UNIFIwifiName")]
    pub unifiwifi_name: String,
}
