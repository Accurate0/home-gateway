use rand::{RngExt, distr::Alphanumeric};

use super::hash_key;

const KEY_PREFIX: &str = "hg_";
const KEY_RANDOM_LEN: usize = 40;
const KEY_PREFIX_RANDOM_LEN: usize = 6;

pub struct GeneratedKey {
    pub key: String,
    pub key_prefix: String,
    pub key_hash: String,
}

impl GeneratedKey {
    pub fn generate() -> Self {
        let random: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(KEY_RANDOM_LEN)
            .map(char::from)
            .collect();

        let key = format!("{KEY_PREFIX}{random}");
        let key_prefix = format!("{KEY_PREFIX}{}", &random[..KEY_PREFIX_RANDOM_LEN]);
        let key_hash = hash_key(&key);

        Self {
            key,
            key_prefix,
            key_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_key_hashes_to_its_stored_hash_and_starts_with_its_prefix() {
        let generated = GeneratedKey::generate();

        assert_eq!(generated.key.len(), KEY_PREFIX.len() + KEY_RANDOM_LEN);
        assert!(generated.key.starts_with(&generated.key_prefix));
        assert_eq!(generated.key_hash, hash_key(&generated.key));
    }
}
