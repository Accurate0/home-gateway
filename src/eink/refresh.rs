use bytes::Bytes;

use crate::eink::manager::decision::WakeDecision;
use crate::eink::panel::{PACKED_FRAME_SIZE, PartialWindow, crop_packed};

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PlannedRefresh {
    Unchanged,
    Clear,
    Full { image: Bytes },
    Partial { window: PartialWindow, image: Bytes },
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RefreshError {
    #[error("planned frame has the wrong size")]
    WrongFrameSize,
}

impl PlannedRefresh {
    pub fn image_bytes(&self) -> usize {
        match self {
            PlannedRefresh::Full { image } | PlannedRefresh::Partial { image, .. } => image.len(),
            PlannedRefresh::Clear | PlannedRefresh::Unchanged => 0,
        }
    }
}

pub fn planned_refresh(
    decision: &WakeDecision,
    displayed: Option<&str>,
) -> Result<PlannedRefresh, RefreshError> {
    if decision.clear_screen {
        return Ok(PlannedRefresh::Clear);
    }

    let Some(frame) = &decision.frame else {
        return Ok(PlannedRefresh::Unchanged);
    };

    if displayed == Some(frame.hash.as_str()) {
        return Ok(PlannedRefresh::Unchanged);
    }

    if frame.packed.len() != PACKED_FRAME_SIZE {
        tracing::error!(
            hash = %frame.hash,
            len = frame.packed.len(),
            "planned frame is not {PACKED_FRAME_SIZE} bytes"
        );
        return Err(RefreshError::WrongFrameSize);
    }

    Ok(match frame.partial {
        Some(window) => PlannedRefresh::Partial {
            window,
            image: Bytes::from(crop_packed(&frame.packed, window)),
        },
        None => PlannedRefresh::Full {
            image: frame.packed.clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::decision::PlannedFrame;
    use pretty_assertions::assert_eq;

    const HASH: &str = "aa00000000000000000000000000000000000000000000000000000000000001";
    const OTHER_HASH: &str = "bb00000000000000000000000000000000000000000000000000000000000002";

    fn decision(partial: Option<PartialWindow>) -> WakeDecision {
        WakeDecision {
            refresh_secs: 900,
            firmware_version: None,
            clear_screen: false,
            frame: Some(PlannedFrame {
                hash: HASH.to_owned(),
                packed: bytes::Bytes::from(vec![0x11; PACKED_FRAME_SIZE]),
                partial,
            }),
        }
    }

    #[test]
    fn a_display_already_showing_the_frame_gets_no_refresh() {
        let planned = planned_refresh(&decision(None), Some(HASH)).unwrap();

        assert_eq!(planned, PlannedRefresh::Unchanged);
    }

    #[test]
    fn a_display_with_no_known_frame_gets_the_full_frame() {
        let planned = planned_refresh(&decision(None), None).unwrap();

        assert!(matches!(planned, PlannedRefresh::Full { .. }));
        assert_eq!(planned.image_bytes(), PACKED_FRAME_SIZE);
    }

    #[test]
    fn a_partial_window_sends_only_the_cropped_region() {
        let window = PartialWindow {
            x: 0,
            y: 0,
            width: 64,
            height: 8,
        };

        let planned = planned_refresh(&decision(Some(window)), Some(OTHER_HASH)).unwrap();

        assert!(matches!(planned, PlannedRefresh::Partial { window: sent, .. } if sent == window));
        assert_eq!(planned.image_bytes(), 64 / 2 * 8);
    }

    #[test]
    fn clear_screen_wins_over_the_frame() {
        let clearing = WakeDecision {
            clear_screen: true,
            ..decision(None)
        };

        let planned = planned_refresh(&clearing, Some(HASH)).unwrap();

        assert_eq!(planned, PlannedRefresh::Clear);
    }

    #[test]
    fn no_planned_frame_leaves_the_panel_alone() {
        let empty = WakeDecision {
            frame: None,
            ..decision(None)
        };

        let planned = planned_refresh(&empty, None).unwrap();

        assert_eq!(planned, PlannedRefresh::Unchanged);
    }

    #[test]
    fn a_truncated_frame_is_an_error_rather_than_a_corrupt_draw() {
        let truncated = WakeDecision {
            frame: Some(PlannedFrame {
                hash: HASH.to_owned(),
                packed: bytes::Bytes::from(vec![0x11; 10]),
                partial: None,
            }),
            ..decision(None)
        };

        assert_eq!(
            planned_refresh(&truncated, None),
            Err(RefreshError::WrongFrameSize)
        );
    }
}
