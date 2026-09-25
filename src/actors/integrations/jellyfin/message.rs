use crate::integrations::jellyfin::types::Session;

pub enum JellyfinMessage {
    Poll,
    Snapshot(Vec<Session>),
}
