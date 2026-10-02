use super::EinkDisplayManager;
use super::decision::WakeDecision;
use super::resolve::ResolvedDisplay;
use crate::routes::epd::{DeviceReport, EpdConfig};

#[cfg(debug_assertions)]
const HOST: &str = "http://192.168.0.149:8000/v1/epd";
#[cfg(not(debug_assertions))]
const HOST: &str = "https://home.anurag.sh/v1/epd";

pub fn firmware_url(device_id: &str) -> String {
    format!("{HOST}/firmware?device_id={device_id}")
}

impl EinkDisplayManager {
    #[tracing::instrument(
        name = "eink.epd_config",
        skip_all,
        fields(device_id = %resolved.device_id)
    )]
    pub async fn epd_config(
        &self,
        resolved: &ResolvedDisplay,
        report: DeviceReport<'_>,
    ) -> EpdConfig {
        let decision = self.wake_decision(resolved, report).await;

        epd_config_from(&resolved.device_id, decision)
    }
}

fn epd_config_from(device_id: &str, decision: WakeDecision) -> EpdConfig {
    let mut config = EpdConfig {
        refresh_interval_mins: Some(decision.refresh_secs.div_ceil(60)),
        refresh_interval_secs: Some(decision.refresh_secs),
        image_url: None,
        image_hash: None,
        clear_screen: Some(decision.clear_screen),
        firmware_url: None,
        firmware_version: None,
        partial: None,
    };

    if let Some(version) = decision.firmware_version {
        config.firmware_version = Some(version);
        config.firmware_url = Some(firmware_url(device_id));
    }

    let Some(frame) = decision.frame else {
        return config;
    };

    let mut url = format!("{HOST}/image/{}?device_id={device_id}", frame.hash);

    if let Some(window) = frame.partial {
        url.push_str(&format!(
            "&x={}&y={}&width={}&height={}",
            window.x, window.y, window.width, window.height
        ));
    }

    config.image_url = Some(url);
    config.image_hash = Some(frame.hash);
    config.partial = frame.partial;

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::decision::PlannedFrame;
    use crate::eink::panel::PartialWindow;
    use pretty_assertions::assert_eq;

    const HASH: &str = "aa00000000000000000000000000000000000000000000000000000000000001";

    fn decision(partial: Option<PartialWindow>) -> WakeDecision {
        WakeDecision {
            refresh_secs: 90,
            firmware_version: Some("v2".to_owned()),
            clear_screen: false,
            frame: Some(PlannedFrame {
                hash: HASH.to_owned(),
                packed: bytes::Bytes::new(),
                partial,
            }),
        }
    }

    #[test]
    fn a_full_frame_links_the_image_and_firmware() {
        let config = epd_config_from("panel", decision(None));

        assert_eq!(config.refresh_interval_secs, Some(90));
        assert_eq!(config.refresh_interval_mins, Some(2));
        assert_eq!(config.image_hash.as_deref(), Some(HASH));
        assert_eq!(
            config.image_url,
            Some(format!("{HOST}/image/{HASH}?device_id=panel"))
        );
        assert_eq!(config.firmware_url, Some(firmware_url("panel")));
        assert_eq!(config.firmware_version.as_deref(), Some("v2"));
    }

    #[test]
    fn a_partial_frame_carries_its_window_in_the_url() {
        let window = PartialWindow {
            x: 16,
            y: 8,
            width: 64,
            height: 32,
        };

        let config = epd_config_from("panel", decision(Some(window)));

        assert_eq!(
            config.image_url,
            Some(format!(
                "{HOST}/image/{HASH}?device_id=panel&x=16&y=8&width=64&height=32"
            ))
        );
        assert!(config.partial.is_some());
    }

    #[test]
    fn no_frame_leaves_the_image_unset() {
        let config = epd_config_from(
            "panel",
            WakeDecision {
                refresh_secs: 60,
                firmware_version: None,
                clear_screen: true,
                frame: None,
            },
        );

        assert_eq!(config.image_url, None);
        assert_eq!(config.image_hash, None);
        assert_eq!(config.firmware_url, None);
        assert_eq!(config.clear_screen, Some(true));
    }
}
