use super::manager::resolve::ResolvedDisplay;
use super::panel::{PartialWindow, dirty_window};

#[tracing::instrument(
    name = "eink.partial_window",
    skip_all,
    fields(device_id = tracing::field::Empty)
)]
pub async fn resolve_partial_window(
    eink: &crate::repo::EinkRepo,
    display: &ResolvedDisplay,
    current_image_hash: Option<&str>,
    new_hash: &str,
    previous_packed: Option<&[u8]>,
    new_packed: &[u8],
) -> Option<PartialWindow> {
    let device_id = display.device_id.as_str();
    tracing::Span::current().record("device_id", device_id);

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

    let window = match previous_packed {
        Some(previous) => {
            tracing::info_span!("eink.dirty_window").in_scope(|| dirty_window(previous, new_packed))
        }
        None => None,
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
