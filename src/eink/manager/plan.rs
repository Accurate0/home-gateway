use super::frame::FrameContext;
use crate::settings::SleepWindow;
use sha2::{Digest, Sha256};

pub struct RenderPlan {
    pub image_key: String,
    pub sleep: Option<SleepWindow>,
    pub hash: String,
    pub frame: FrameContext,
}

pub fn render_hash(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }

    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_hash_is_stable() {
        assert_eq!(
            render_hash(&["1", "key.png"]),
            render_hash(&["1", "key.png"])
        );
    }

    #[test]
    fn render_hash_changes_with_inputs() {
        let base = render_hash(&["1", "key.png", "false"]);

        assert_ne!(base, render_hash(&["2", "key.png", "false"]));
        assert_ne!(base, render_hash(&["1", "other.png", "false"]));
        assert_ne!(base, render_hash(&["1", "key.png", "true"]));
    }

    #[test]
    fn render_hash_separates_parts() {
        assert_ne!(render_hash(&["ab", "c"]), render_hash(&["a", "bc"]));
    }
}
