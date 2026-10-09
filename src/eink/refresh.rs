use tonic::Status;

use crate::eink::manager::decision::WakeDecision;
use crate::eink::panel::{PACKED_FRAME_SIZE, crop_packed};
use crate::grpc::proto::wake_response::Refresh;
use crate::grpc::proto::{Clear, FullFrame, PartialFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Outcome {
    Unchanged,
    Clear,
    Full,
    Partial,
}

pub struct PlannedRefresh {
    pub outcome: Outcome,
    pub refresh: Option<Refresh>,
}

pub fn planned_refresh(
    decision: &WakeDecision,
    displayed: Option<&str>,
) -> Result<PlannedRefresh, Status> {
    if decision.clear_screen {
        return Ok(PlannedRefresh {
            outcome: Outcome::Clear,
            refresh: Some(Refresh::Clear(Clear {})),
        });
    }

    let Some(frame) = &decision.frame else {
        return Ok(PlannedRefresh {
            outcome: Outcome::Unchanged,
            refresh: None,
        });
    };

    if displayed == Some(frame.hash.as_str()) {
        return Ok(PlannedRefresh {
            outcome: Outcome::Unchanged,
            refresh: None,
        });
    }

    if frame.packed.len() != PACKED_FRAME_SIZE {
        tracing::error!(
            hash = %frame.hash,
            len = frame.packed.len(),
            "planned frame is not {PACKED_FRAME_SIZE} bytes"
        );
        return Err(Status::internal("planned frame has the wrong size"));
    }

    Ok(match frame.partial {
        Some(window) => PlannedRefresh {
            outcome: Outcome::Partial,
            refresh: Some(Refresh::Partial(PartialFrame {
                x: window.x,
                y: window.y,
                width: window.width,
                height: window.height,
                image: bytes::Bytes::from(crop_packed(&frame.packed, window)),
            })),
        },
        None => PlannedRefresh {
            outcome: Outcome::Full,
            refresh: Some(Refresh::Full(FullFrame {
                image: frame.packed.clone(),
            })),
        },
    })
}

pub fn image_bytes(refresh: Option<&Refresh>) -> usize {
    match refresh {
        Some(Refresh::Full(frame)) => frame.image.len(),
        Some(Refresh::Partial(frame)) => frame.image.len(),
        Some(Refresh::Clear(_)) | None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::decision::PlannedFrame;
    use crate::eink::panel::PartialWindow;
    use pretty_assertions::assert_eq;
    use tonic::Code;

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

        assert_eq!(planned.outcome, Outcome::Unchanged);
        assert!(planned.refresh.is_none());
    }

    #[test]
    fn a_display_with_no_known_frame_gets_the_full_frame() {
        let planned = planned_refresh(&decision(None), None).unwrap();

        assert_eq!(planned.outcome, Outcome::Full);
        assert_eq!(image_bytes(planned.refresh.as_ref()), PACKED_FRAME_SIZE);
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

        assert_eq!(planned.outcome, Outcome::Partial);
        assert_eq!(image_bytes(planned.refresh.as_ref()), 64 / 2 * 8);
    }

    #[test]
    fn clear_screen_wins_over_the_frame() {
        let clearing = WakeDecision {
            clear_screen: true,
            ..decision(None)
        };

        let planned = planned_refresh(&clearing, Some(HASH)).unwrap();

        assert_eq!(planned.outcome, Outcome::Clear);
        assert!(matches!(planned.refresh, Some(Refresh::Clear(_))));
    }

    #[test]
    fn no_planned_frame_leaves_the_panel_alone() {
        let empty = WakeDecision {
            frame: None,
            ..decision(None)
        };

        let planned = planned_refresh(&empty, None).unwrap();

        assert_eq!(planned.outcome, Outcome::Unchanged);
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

        let status = planned_refresh(&truncated, None).err().unwrap();

        assert_eq!(status.code(), Code::Internal);
    }
}
