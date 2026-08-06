use super::panel::{PartialWindow, dirty_window, packed_cache_key};
use crate::device_registry::DeviceRegistry;
use super::flag::EpdFlagConfig;

pub async fn resolve_partial_window(
    db: &sqlx::Pool<sqlx::Postgres>,
    s3: &crate::s3::S3,
    devices: &DeviceRegistry,
    flag: &EpdFlagConfig,
    device_id: &str,
    current_image_hash: Option<&str>,
    new_hash: &str,
) -> Option<PartialWindow> {
    let display = devices.eink_display(device_id)?;
    let policy = display.partial;

    if !flag.partial_refresh.unwrap_or(policy.enabled) {
        return None;
    }

    let current_image_hash = current_image_hash?;

    if current_image_hash == new_hash {
        return None;
    }

    let consecutive = partial_refresh_count(db, device_id).await;
    if consecutive >= policy.max_consecutive {
        tracing::info!(
            device_id = %device_id,
            consecutive,
            "forcing a full refresh to clear accumulated ghosting"
        );
        reset_partial_refresh_count(db, device_id).await;
        return None;
    }

    let previous = s3
        .get_object(&packed_cache_key(current_image_hash.to_owned()))
        .await
        .ok();
    let next = s3
        .get_object(&packed_cache_key(new_hash.to_owned()))
        .await
        .ok();

    let window = match (previous.as_deref(), next.as_deref()) {
        (Some(previous), Some(next)) => dirty_window(previous, next),
        _ => None,
    };

    let Some(window) = window else {
        reset_partial_refresh_count(db, device_id).await;
        return None;
    };

    let area_pct = window.area_pct();

    if area_pct > policy.max_area_pct {
        tracing::info!(
            device_id = %device_id,
            area_pct,
            "dirty region too large for a partial refresh"
        );
        reset_partial_refresh_count(db, device_id).await;
        return None;
    }

    tracing::info!(
        device_id = %device_id,
        area_pct,
        x = window.x,
        y = window.y,
        width = window.width,
        height = window.height,
        "serving a partial refresh"
    );

    increment_partial_refresh_count(db, device_id).await;

    Some(window)
}

async fn partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) -> i32 {
    let result = sqlx::query_scalar!(
        "SELECT partial_refresh_count FROM eink_display WHERE device_id = $1",
        device_id
    )
    .fetch_optional(db)
    .await;

    match result {
        Ok(count) => count.unwrap_or(0),
        Err(e) => {
            tracing::warn!(device_id = %device_id, "failed to read partial refresh count: {e}");
            0
        }
    }
}

async fn increment_partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) {
    let result = sqlx::query!(
        "UPDATE eink_display SET partial_refresh_count = partial_refresh_count + 1 WHERE device_id = $1",
        device_id
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::warn!(device_id = %device_id, "failed to increment partial refresh count: {e}");
    }
}

async fn reset_partial_refresh_count(db: &sqlx::Pool<sqlx::Postgres>, device_id: &str) {
    let result = sqlx::query!(
        "UPDATE eink_display SET partial_refresh_count = 0 WHERE device_id = $1",
        device_id
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::warn!(device_id = %device_id, "failed to reset partial refresh count: {e}");
    }
}
