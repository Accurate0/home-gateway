use mlua::ExternalError;

use crate::lua::{LuaCallContext, lua_module};

use super::types::{Forecast, ForecastDetails};

pub struct WeatherLua;

#[lua_module(namespace = "weather")]
impl WeatherLua {
    #[lua(scope = Weather::Read)]
    async fn forecast(
        cx: &LuaCallContext,
        location: Option<String>,
    ) -> mlua::Result<Option<Forecast>> {
        fetch(cx, location.as_deref()).await
    }

    #[lua(scope = Weather::Read)]
    async fn today(
        cx: &LuaCallContext,
        location: Option<String>,
    ) -> mlua::Result<Option<ForecastDetails>> {
        let forecast = fetch(cx, location.as_deref()).await?;

        Ok(forecast.and_then(|forecast| forecast.days.into_iter().next()))
    }
}

async fn fetch(cx: &LuaCallContext, location: Option<&str>) -> mlua::Result<Option<Forecast>> {
    let settings = &cx.state.settings.integrations.willyweather;
    let requested = location.unwrap_or(&settings.default_location);

    let Some(alias) = settings.resolve_location(requested) else {
        return Err(format!(
            "unknown weather location `{requested}`; available: [{}]",
            settings
                .locations
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
        .into_lua_err());
    };

    let alias = alias.to_owned();

    let forecast = cx
        .query(WeatherLua::FORECAST, || async {
            cx.state.repos.willyweather().forecast(&alias).await
        })
        .await?;

    if forecast.is_none() {
        tracing::info!(
            "[{}] lua weather has no stored forecast for {alias}",
            cx.event_id
        );
    }

    Ok(forecast)
}
