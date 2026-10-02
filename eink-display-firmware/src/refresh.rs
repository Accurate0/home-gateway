use bytes::Bytes;

use crate::driver::EPD_IMAGE_FULL_BUFFER_SIZE;
use crate::proto::wake_response;

#[derive(Debug, Clone, Copy)]
pub struct PartialWindow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PartialWindow {
    pub fn buffer_size(&self) -> usize {
        (self.width / 2) as usize * self.height as usize
    }
}

pub enum Refresh {
    Clear,
    Image(Bytes),
    Partial { window: PartialWindow, image: Bytes },
}

impl Refresh {
    pub fn from_response(refresh: wake_response::Refresh) -> Option<Self> {
        match refresh {
            wake_response::Refresh::Clear(_) => Some(Refresh::Clear),

            wake_response::Refresh::Full(frame) => {
                if frame.image.len() != EPD_IMAGE_FULL_BUFFER_SIZE {
                    log::error!(
                        "full frame is {} bytes, expected {}",
                        frame.image.len(),
                        EPD_IMAGE_FULL_BUFFER_SIZE
                    );
                    return None;
                }

                Some(Refresh::Image(frame.image))
            }

            wake_response::Refresh::Partial(frame) => {
                let window = PartialWindow {
                    x: frame.x,
                    y: frame.y,
                    width: frame.width,
                    height: frame.height,
                };

                log::info!(
                    "partial refresh requested: x={} y={} w={} h={}",
                    window.x,
                    window.y,
                    window.width,
                    window.height
                );

                if frame.image.len() != window.buffer_size() {
                    log::error!(
                        "partial frame is {} bytes, its window needs {}",
                        frame.image.len(),
                        window.buffer_size()
                    );
                    return None;
                }

                Some(Refresh::Partial {
                    window,
                    image: frame.image,
                })
            }
        }
    }
}

pub fn describe(refresh: Option<&wake_response::Refresh>) -> &'static str {
    match refresh {
        Some(wake_response::Refresh::Clear(_)) => "clear",
        Some(wake_response::Refresh::Full(_)) => "full",
        Some(wake_response::Refresh::Partial(_)) => "partial",
        None => "unchanged",
    }
}
