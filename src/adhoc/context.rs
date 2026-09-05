use sqlx::{Postgres, Transaction};

use crate::device_registry::DeviceRegistry;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::jellyfin::Jellyfin;
use crate::integrations::s3::S3;
use crate::settings::SettingsContainer;
use crate::state::AppState;

pub struct AdhocTaskContext<'a> {
    pub tx: &'a mut Transaction<'static, Postgres>,
    pub devices: &'a DeviceRegistry,
    pub settings: &'a SettingsContainer,
    pub s3: &'a S3,
    pub home_assistant: Option<&'a HomeAssistant>,
    pub jellyfin: Option<&'a Jellyfin>,
}

impl<'a> AdhocTaskContext<'a> {
    pub fn new(state: &'a AppState, tx: &'a mut Transaction<'static, Postgres>) -> Self {
        Self {
            tx,
            devices: &state.devices,
            settings: &state.settings,
            s3: state.handles.expect::<S3>(),
            home_assistant: state.handles.get::<HomeAssistant>(),
            jellyfin: state.handles.get::<Jellyfin>(),
        }
    }
}
