use serde::{Deserialize, Serialize};

pub const PANEL_WIDTH: u32 = 1200;
pub const PANEL_HEIGHT: u32 = 1600;
const PANEL_HALF_WIDTH: u32 = PANEL_WIDTH / 2;
const WINDOW_X_ALIGN: u32 = 4;
pub const WINDOW_MIN_WIDTH: u32 = 16;

const PACKED_CACHE_PREFIX: &str = "eink-display/packed/";
pub const PACKED_ROW_BYTES: usize = 600;
pub const PACKED_FRAME_HEIGHT: usize = 1600;
pub const PACKED_FRAME_SIZE: usize = PACKED_ROW_BYTES * PACKED_FRAME_HEIGHT;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, async_graphql::SimpleObject,
)]
#[graphql(rename_fields = "camelCase")]
pub struct PartialWindow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PartialWindow {
    pub fn is_valid(&self) -> bool {
        self.width >= WINDOW_MIN_WIDTH
            && self.height >= 2
            && self.x.is_multiple_of(WINDOW_X_ALIGN)
            && self.width.is_multiple_of(WINDOW_X_ALIGN)
            && self.y.is_multiple_of(2)
            && self.height.is_multiple_of(2)
            && self.x + self.width <= PANEL_WIDTH
            && self.y + self.height <= PANEL_HEIGHT
    }

    pub fn area_pct(&self) -> u32 {
        (self.width as u64 * self.height as u64 * 100 / (PANEL_WIDTH as u64 * PANEL_HEIGHT as u64))
            as u32
    }
}

pub fn packed_cache_key(hash: String) -> String {
    format!("{PACKED_CACHE_PREFIX}{hash}.bin")
}

pub fn dirty_window(previous: &[u8], next: &[u8]) -> Option<PartialWindow> {
    if previous.len() != PACKED_FRAME_SIZE || next.len() != PACKED_FRAME_SIZE {
        return None;
    }

    let mut min_byte = PACKED_ROW_BYTES;
    let mut max_byte = 0usize;
    let mut min_row = PACKED_FRAME_HEIGHT;
    let mut max_row = 0usize;

    for row in 0..PACKED_FRAME_HEIGHT {
        let start = row * PACKED_ROW_BYTES;
        let previous_row = &previous[start..start + PACKED_ROW_BYTES];
        let next_row = &next[start..start + PACKED_ROW_BYTES];

        if previous_row == next_row {
            continue;
        }

        let first = previous_row
            .iter()
            .zip(next_row)
            .position(|(a, b)| a != b)
            .unwrap_or(0);
        let last = previous_row
            .iter()
            .zip(next_row)
            .rposition(|(a, b)| a != b)
            .unwrap_or(PACKED_ROW_BYTES - 1);

        min_byte = min_byte.min(first);
        max_byte = max_byte.max(last);
        min_row = min_row.min(row);
        max_row = max_row.max(row);
    }

    if min_row > max_row {
        return None;
    }

    let x0 = (min_byte as u32 * 2) / WINDOW_X_ALIGN * WINDOW_X_ALIGN;
    let x1 = ((max_byte as u32 + 1) * 2).div_ceil(WINDOW_X_ALIGN) * WINDOW_X_ALIGN;
    let y0 = min_row as u32 & !1;
    let y1 = (max_row as u32 + 1).div_ceil(2) * 2;

    let mut window = PartialWindow {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    };

    if window.width < WINDOW_MIN_WIDTH {
        window.width = WINDOW_MIN_WIDTH;
        window.x = window.x.min(PANEL_WIDTH - WINDOW_MIN_WIDTH);
    }

    if window.x < PANEL_HALF_WIDTH && window.x + window.width > PANEL_HALF_WIDTH {
        let left = PANEL_HALF_WIDTH - window.x;
        let right = window.x + window.width - PANEL_HALF_WIDTH;

        if left < WINDOW_MIN_WIDTH {
            window.x = PANEL_HALF_WIDTH - WINDOW_MIN_WIDTH;
            window.width = right + WINDOW_MIN_WIDTH;
        }

        if right < WINDOW_MIN_WIDTH {
            window.width = PANEL_HALF_WIDTH - window.x + WINDOW_MIN_WIDTH;
        }
    }

    window.is_valid().then_some(window)
}

pub fn crop_packed(packed: &[u8], window: PartialWindow) -> Vec<u8> {
    let row_offset = (window.x / 2) as usize;
    let row_count = (window.width / 2) as usize;

    let mut cropped = Vec::with_capacity(row_count * window.height as usize);
    for row in window.y..window.y + window.height {
        let start = row as usize * PACKED_ROW_BYTES + row_offset;
        cropped.extend_from_slice(&packed[start..start + row_count]);
    }

    cropped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_frame() -> Vec<u8> {
        vec![0x11; PACKED_FRAME_SIZE]
    }

    fn set_pixel(frame: &mut [u8], x: usize, y: usize) {
        frame[y * PACKED_ROW_BYTES + x / 2] = 0x00;
    }

    #[test]
    fn an_unchanged_frame_has_no_dirty_window() {
        assert_eq!(dirty_window(&blank_frame(), &blank_frame()), None);
    }

    #[test]
    fn a_wrongly_sized_frame_has_no_dirty_window() {
        assert_eq!(dirty_window(&[0u8; 8], &[1u8; 8]), None);
    }

    #[test]
    fn a_single_changed_pixel_yields_the_minimum_window() {
        let previous = blank_frame();
        let mut next = previous.clone();
        set_pixel(&mut next, 400, 500);

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert!(window.is_valid());
        assert_eq!(window.width, WINDOW_MIN_WIDTH);
        assert_eq!(window.height, 2);
        assert_eq!(window.y, 500);
        assert!(window.x <= 400 && 400 < window.x + window.width);
    }

    #[test]
    fn a_change_straddling_the_controller_split_covers_both_halves() {
        let previous = blank_frame();
        let mut next = previous.clone();
        set_pixel(&mut next, 598, 100);
        set_pixel(&mut next, 602, 100);

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert!(window.is_valid());
        assert!(window.x + WINDOW_MIN_WIDTH <= PANEL_HALF_WIDTH);
        assert!(window.x + window.width >= PANEL_HALF_WIDTH + WINDOW_MIN_WIDTH);
    }

    #[test]
    fn a_full_frame_change_covers_the_whole_panel() {
        let previous = blank_frame();
        let next = vec![0x00; PACKED_FRAME_SIZE];

        let window = dirty_window(&previous, &next).expect("a dirty window");

        assert_eq!(
            window,
            PartialWindow {
                x: 0,
                y: 0,
                width: PANEL_WIDTH,
                height: PANEL_HEIGHT,
            }
        );
        assert_eq!(window.area_pct(), 100);
    }

    #[test]
    fn a_crop_matches_the_same_region_of_the_full_frame() {
        let mut packed = blank_frame();
        for (index, byte) in packed.iter_mut().enumerate() {
            *byte = index as u8;
        }

        let window = PartialWindow {
            x: 400,
            y: 500,
            width: 64,
            height: 4,
        };
        let cropped = crop_packed(&packed, window);

        assert_eq!(cropped.len(), window.width as usize / 2 * 4);
        for row in 0..window.height as usize {
            let source = (500 + row) * PACKED_ROW_BYTES + 200;
            let target = row * 32;
            assert_eq!(&cropped[target..target + 32], &packed[source..source + 32]);
        }
    }
}
