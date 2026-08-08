use crate::{
    device_registry::DeviceRegistry,
    integrations::feature_flag::FeatureFlagClient,
    settings::{EinkMode, PaletteColor, RedditTimespan},
};
use open_feature::EvaluationContext;

const EPD_FLAG: &str = "home-gateway-epd";
const EPD_DITHERING_CONFIG_FLAG: &str = "home-gateway-epd-dithering-config";

#[derive(Debug, Clone, Default)]
pub struct EpdFlagConfig {
    pub clear_screen: bool,
    pub force_sleep: bool,
    pub refresh_interval: Option<u32>,
    pub mode: Option<EinkMode>,
    pub dashboard_view: Option<String>,
    pub album: Option<String>,
    pub subreddit: Option<String>,
    pub timespan: Option<RedditTimespan>,
    pub limit: Option<u32>,
    pub firmware_version: Option<String>,
    pub partial_refresh: Option<bool>,
}

impl From<open_feature::StructValue> for EpdFlagConfig {
    fn from(value: open_feature::StructValue) -> Self {
        let default = EpdFlagConfig::default();
        let string_field = |key: &str| {
            value
                .fields
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        };
        Self {
            clear_screen: value
                .fields
                .get("clear_screen")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.clear_screen),
            force_sleep: value
                .fields
                .get("force_sleep")
                .and_then(|v| v.as_bool())
                .unwrap_or(default.force_sleep),
            refresh_interval: value
                .fields
                .get("refresh_interval")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|n| n.max(1) as u32),
            mode: string_field("mode").and_then(|s| match s.as_str() {
                "dashboard" => Some(EinkMode::Dashboard),
                "album" => Some(EinkMode::Album),
                "reddit" => Some(EinkMode::Reddit),
                other => {
                    tracing::warn!("unknown epd mode `{other}` in flag, ignoring");
                    None
                }
            }),
            dashboard_view: string_field("dashboard_view"),
            album: string_field("album"),
            subreddit: string_field("subreddit"),
            timespan: string_field("timespan").and_then(|s| match s.as_str() {
                "hour" => Some(RedditTimespan::Hour),
                "day" => Some(RedditTimespan::Day),
                "week" => Some(RedditTimespan::Week),
                "month" => Some(RedditTimespan::Month),
                "year" => Some(RedditTimespan::Year),
                "all" => Some(RedditTimespan::All),
                other => {
                    tracing::warn!("unknown reddit timespan `{other}` in flag, ignoring");
                    None
                }
            }),
            limit: value
                .fields
                .get("limit")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|n| n.clamp(1, 100) as u32),
            firmware_version: string_field("firmware_version"),
            partial_refresh: value
                .fields
                .get("partial_refresh")
                .and_then(|v| v.as_bool()),
        }
    }
}

fn evaluation_context(devices: &DeviceRegistry, device_id: &str) -> EvaluationContext {
    let context =
        EvaluationContext::default().with_custom_field("device_id", device_id.to_string());

    match devices.eink_display(device_id) {
        Some(display) => context.with_custom_field("device_name", display.name.clone()),
        None => context,
    }
}

pub async fn epd_flag_config(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    device_id: &str,
) -> EpdFlagConfig {
    match feature_flag_client
        .get_struct(EPD_FLAG, evaluation_context(devices, device_id))
        .await
    {
        Ok(value) => {
            let config = EpdFlagConfig::from(value);
            tracing::info!("{EPD_FLAG} evaluated for {device_id}: {config:?}");
            config
        }
        Err(e) => {
            let fallback = EpdFlagConfig::default();
            tracing::error!(
                "error evaluating {EPD_FLAG} for {device_id}: {e:?}, using fallback: {fallback:?}"
            );
            fallback
        }
    }
}

pub async fn epd_palette(
    feature_flag_client: &FeatureFlagClient,
    devices: &DeviceRegistry,
    base: &[PaletteColor],
    device_id: &str,
) -> Vec<(f32, f32, f32, u8)> {
    let config = feature_flag_client
        .get_struct(
            EPD_DITHERING_CONFIG_FLAG,
            evaluation_context(devices, device_id),
        )
        .await
        .ok();

    base.iter()
        .map(|color| {
            let (r, g, b) = config
                .as_ref()
                .and_then(|c| c.fields.get(color.name))
                .and_then(|v| v.as_struct())
                .map(|s| {
                    let channel = |key: &str, default: f32| {
                        s.fields
                            .get(key)
                            .and_then(|v| {
                                v.as_i64()
                                    .map(|n| n as f32)
                                    .or_else(|| v.as_f64().map(|f| f as f32))
                            })
                            .map(|n| n.clamp(0.0, 255.0))
                            .unwrap_or(default)
                    };
                    (
                        channel("r", color.r),
                        channel("g", color.g),
                        channel("b", color.b),
                    )
                })
                .unwrap_or((color.r, color.g, color.b));
            (r, g, b, color.index)
        })
        .collect()
}
