use std::sync::Arc;

use crate::actors::health::ActorHealthRegistry;
use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::{AuthManager, OAuthValidator};
use crate::eink::EinkDisplayManager;
use crate::http::get_traced_http_client;
use crate::http::public_client::PublicHttpClient;
use crate::integrations::esphome_native_api::{EsphomeNativeApi, Node};
use crate::integrations::{
    feature_flag::FeatureFlagClient,
    fuelwatch::FuelWatch,
    home_assistant::HomeAssistant,
    mqtt::{Mqtt, MqttClient},
    reddit::Reddit,
    s3::S3,
    solar::goodwe::GoodWeSemsAPI,
    transperth::Transperth,
    willyweather::WillyWeather,
};
use crate::settings::HttpClientKind;
use crate::state::HandleRegistry;

use super::Storage;

pub struct Handles {
    pub registry: HandleRegistry,
    pub mqtt: Mqtt,
    pub esphome_nodes: Vec<Node>,
}

pub async fn build(
    storage: &Storage,
    feature_flag_client: &FeatureFlagClient,
) -> anyhow::Result<Handles> {
    let settings = &storage.settings;
    let pool = &storage.pool;

    let http = &settings.http.clients;

    let workflow_manager = WorkflowManager::new(pool.clone(), &settings.workflow.enabled_cache);

    let (mqtt_client, mqtt) = Mqtt::new(&settings.mqtt).await?;

    let integrations = &settings.integrations;

    let s3 = S3::new(
        &integrations.s3.bucket,
        &integrations.s3.region,
        integrations.s3.endpoint.clone(),
    )?;

    let eink = EinkDisplayManager::new(
        pool.clone(),
        s3.clone(),
        feature_flag_client.clone(),
        storage.devices.clone(),
        settings.clone(),
        Reddit::new(http.timeout_for(HttpClientKind::Reddit)),
    );

    let home_assistant = HomeAssistant::from_settings(
        &settings.home_assistant,
        http.timeout_for(HttpClientKind::HomeAssistant),
    );

    let transperth = integrations
        .transperth
        .state
        .is_enabled()
        .then(|| {
            Transperth::new(
                &integrations.transperth,
                http.timeout_for(HttpClientKind::Transperth),
            )
        })
        .transpose()?;

    let http_client = get_traced_http_client(http.timeout())?;
    let public_http_client = PublicHttpClient::new(http.timeout())?;

    let willyweather = WillyWeather::new(
        &integrations.willyweather,
        http.timeout_for(HttpClientKind::WillyWeather),
    )?;

    let fuelwatch = integrations
        .fuelwatch
        .state
        .is_enabled()
        .then(|| {
            FuelWatch::new(
                &integrations.fuelwatch,
                http.timeout_for(HttpClientKind::FuelWatch),
            )
        })
        .transpose()?;

    let goodwe = if integrations.solar.state.is_enabled() {
        GoodWeSemsAPI::new(
            pool.clone(),
            &integrations.solar,
            http.timeout_for(HttpClientKind::GoodWe),
        )
    } else {
        None
    };

    let (esphome_native_api, esphome_nodes) = match integrations.esphome.state.is_enabled() {
        true => {
            let (client, nodes) = EsphomeNativeApi::new(
                storage
                    .devices
                    .esphome_native_api_nodes()
                    .cloned()
                    .collect::<Vec<_>>(),
            );

            match client.is_empty() {
                true => (None, Vec::new()),
                false => {
                    tracing::info!("esphome native api enabled for {} node(s)", nodes.len());

                    (Some(client), nodes)
                }
            }
        }
        false => (None, Vec::new()),
    };

    let oauth = settings
        .auth
        .oauth
        .clone()
        .map(|oauth| {
            OAuthValidator::new(oauth, http.timeout_for(HttpClientKind::OAuth)).map(Arc::new)
        })
        .transpose()?;

    let registry = HandleRegistry::builder()
        .insert(mqtt_client)
        .insert(s3)
        .insert(eink)
        .insert(workflow_manager)
        .insert(ActorHealthRegistry::new())
        .insert(AuthManager::new(
            pool.clone(),
            oauth,
            &settings.auth.api_key_cache,
        ))
        .insert(willyweather)
        .insert(http_client)
        .insert(public_http_client)
        .insert_optional(home_assistant)
        .insert_optional(esphome_native_api)
        .insert_optional(transperth)
        .insert_optional(fuelwatch)
        .insert_optional(goodwe)
        .build();

    assert_required(&registry);

    Ok(Handles {
        registry,
        mqtt,
        esphome_nodes,
    })
}

crate::required_handles! {
    MqttClient => "mqtt",
    S3 => "s3",
    EinkDisplayManager => "eink",
    WorkflowManager => "workflows",
    ActorHealthRegistry => "actor health",
    AuthManager => "auth",
    WillyWeather => "willyweather",
    reqwest_middleware::ClientWithMiddleware => "http client",
    PublicHttpClient => "public http client",
}
