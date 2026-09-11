use std::sync::Arc;

use crate::actors::health::ActorHealthRegistry;
use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::{AuthManager, OAuthValidator};
use crate::eink::EinkDisplayManager;
use crate::http::get_traced_http_client;
use crate::integrations::{
    feature_flag::FeatureFlagClient,
    fuelwatch::FuelWatch,
    home_assistant::HomeAssistant,
    jellyfin::Jellyfin,
    mqtt::{Mqtt, MqttClient},
    reddit::Reddit,
    s3::S3,
    solar::goodwe::GoodWeSemsAPI,
    transperth::Transperth,
    willyweather::WillyWeather,
};
use crate::state::HandleRegistry;

use super::Storage;

pub struct Handles {
    pub registry: HandleRegistry,
    pub mqtt: Mqtt,
}

pub async fn build(
    storage: &Storage,
    feature_flag_client: &FeatureFlagClient,
) -> anyhow::Result<Handles> {
    let settings = &storage.settings;
    let pool = &storage.pool;

    let workflow_manager = WorkflowManager::new(pool.clone());

    let (mqtt_client, mqtt) = Mqtt::new(
        settings.mqtt_url.clone(),
        settings.mqtt_port,
        settings.mqtt_username.clone(),
        settings.mqtt_password.clone(),
    )
    .await?;

    let s3 = S3::new(
        &settings.s3.bucket,
        &settings.s3.region,
        settings.s3.endpoint.clone(),
    )?;

    let eink = EinkDisplayManager::new(
        pool.clone(),
        s3.clone(),
        feature_flag_client.clone(),
        storage.devices.clone(),
        settings.clone(),
        Reddit::new(),
    );

    let home_assistant = HomeAssistant::from_settings(&settings.home_assistant);

    let jellyfin = settings.jellyfin.as_ref().and_then(Jellyfin::new);

    let transperth = settings
        .transperth
        .as_ref()
        .map(Transperth::new)
        .transpose()?;

    let http_client = get_traced_http_client()?;

    let willyweather = WillyWeather::new(&settings.willyweather)?;

    let fuelwatch = settings
        .fuelwatch
        .as_ref()
        .map(FuelWatch::new)
        .transpose()?;

    let goodwe = settings
        .solar
        .as_ref()
        .and_then(|solar| GoodWeSemsAPI::new(pool.clone(), solar));

    let oauth = settings
        .oauth
        .clone()
        .map(|oauth| OAuthValidator::new(oauth).map(Arc::new))
        .transpose()?;

    let registry = HandleRegistry::builder()
        .insert(mqtt_client)
        .insert(s3)
        .insert(eink)
        .insert(workflow_manager)
        .insert(ActorHealthRegistry::new())
        .insert(AuthManager::new(pool.clone(), oauth))
        .insert(willyweather)
        .insert(http_client)
        .insert_optional(home_assistant)
        .insert_optional(jellyfin)
        .insert_optional(transperth)
        .insert_optional(fuelwatch)
        .insert_optional(goodwe)
        .build();

    assert_required(&registry);

    Ok(Handles { registry, mqtt })
}

fn assert_required(handles: &HandleRegistry) {
    for (label, present) in [
        ("mqtt", handles.contains::<MqttClient>()),
        ("s3", handles.contains::<S3>()),
        ("eink", handles.contains::<EinkDisplayManager>()),
        ("workflows", handles.contains::<WorkflowManager>()),
        ("actor health", handles.contains::<ActorHealthRegistry>()),
        ("auth", handles.contains::<AuthManager>()),
        ("willyweather", handles.contains::<WillyWeather>()),
        (
            "http client",
            handles.contains::<reqwest_middleware::ClientWithMiddleware>(),
        ),
    ] {
        if !present {
            panic!("the {label} handle was not registered before startup");
        }
    }
}
