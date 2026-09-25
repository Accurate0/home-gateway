mod item;
mod play_state;
mod session;

pub use item::Item;
pub use play_state::PlayState;
pub use session::Session;

const TICKS_PER_SECOND: f64 = 10_000_000.0;

pub fn ticks_to_seconds(ticks: i64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND
}
