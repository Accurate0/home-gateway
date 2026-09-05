use super::manager::resolve::ResolvedDisplay;
use super::panel::{PartialWindow, dirty_window, packed_cache_key};

pub async fn resolve_partial_window(
    eink: &crate::repo::EinkRepo,
    s3: &crate::integrations::s3::S3,
    display: &ResolvedDisplay,
    current_image_hash: Option<&str>,
    new_hash: &str,
) -> Option<PartialWindow> {
    let device_id = display.device_id.as_str();
    let policy = display.partial;

    if !display.partial_enabled {
        return None;
    }

    let current_image_hash = current_image_hash?;

    if current_image_hash == new_hash {
        return None;
    }

    let consecutive = partial_refresh_count(eink, device_id).await;
    if consecutive >= policy.max_consecutive {
        tracing::info!(
            device_id = %device_id,
            consecutive,
            "forcing a full refresh to clear accumulated ghosting"
        );
        reset_partial_refresh_count(eink, device_id).await;
        return None;
    }

    let previous = s3
        .get_object(&packed_cache_key(current_image_hash)?)
        .await
        .ok();
    let next = s3.get_object(&packed_cache_key(new_hash)?).await.ok();

    let window = match (previous.as_deref(), next.as_deref()) {
        (Some(previous), Some(next)) => dirty_window(previous, next),
        _ => None,
    };

    let Some(window) = window else {
        reset_partial_refresh_count(eink, device_id).await;
        return None;
    };

    let area_pct = window.area_pct();

    if area_pct > policy.max_area_pct {
        tracing::info!(
            device_id = %device_id,
            area_pct,
            "dirty region too large for a partial refresh"
        );
        reset_partial_refresh_count(eink, device_id).await;
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

    increment_partial_refresh_count(eink, device_id).await;

    Some(window)
}

async fn partial_refresh_count(eink: &crate::repo::EinkRepo, device_id: &str) -> i32 {
    match eink.partial_refresh_count(device_id).await {
        Ok(count) => count,
        Err(e) => {
            tracing::warn!(device_id = %device_id, "failed to read partial refresh count: {e}");
            0
        }
    }
}

async fn increment_partial_refresh_count(eink: &crate::repo::EinkRepo, device_id: &str) {
    if let Err(e) = eink.increment_partial_refresh_count(device_id).await {
        tracing::warn!(device_id = %device_id, "failed to increment partial refresh count: {e}");
    }
}

async fn reset_partial_refresh_count(eink: &crate::repo::EinkRepo, device_id: &str) {
    if let Err(e) = eink.reset_partial_refresh_count(device_id).await {
        tracing::warn!(device_id = %device_id, "failed to reset partial refresh count: {e}");
    }
}
