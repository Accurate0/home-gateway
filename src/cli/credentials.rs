use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

const REFRESH_LEEWAY_SECS: i64 = 60;

#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
    #[error("could not determine a config directory: set XDG_CONFIG_HOME or HOME")]
    NoConfigDir,
    #[error("failed to read credentials: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse credentials, run `home login` again: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub issuer: String,
    pub client_id: String,
}

impl Credentials {
    pub fn expires_soon(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => expires_at <= Utc::now() + Duration::seconds(REFRESH_LEEWAY_SECS),
            None => false,
        }
    }
}

pub fn path() -> Result<PathBuf, CredentialsError> {
    let base = match std::env::var("XDG_CONFIG_HOME") {
        Ok(value) if !value.is_empty() => PathBuf::from(value),
        _ => PathBuf::from(std::env::var("HOME").map_err(|_| CredentialsError::NoConfigDir)?)
            .join(".config"),
    };

    Ok(base.join("home-gateway").join("credentials.json"))
}

pub fn load() -> Result<Option<Credentials>, CredentialsError> {
    let path = path()?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    Ok(Some(serde_json::from_str(&contents)?))
}

pub fn store(credentials: &Credentials) -> Result<PathBuf, CredentialsError> {
    let path = path()?;

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)?;

    file.write_all(serde_json::to_string_pretty(credentials)?.as_bytes())?;
    file.write_all(b"\n")?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;

    Ok(path)
}

pub fn clear() -> Result<bool, CredentialsError> {
    let path = path()?;

    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials(expires_at: Option<DateTime<Utc>>) -> Credentials {
        Credentials {
            access_token: "token".to_owned(),
            refresh_token: None,
            expires_at,
            issuer: "https://idm.example/oauth2/openid/home-gateway".to_owned(),
            client_id: "home-gateway".to_owned(),
        }
    }

    #[test]
    fn a_token_without_an_expiry_never_expires() {
        assert!(!credentials(None).expires_soon());
    }

    #[test]
    fn a_token_expiring_inside_the_leeway_is_stale() {
        let expires_at = Utc::now() + Duration::seconds(REFRESH_LEEWAY_SECS / 2);
        assert!(credentials(Some(expires_at)).expires_soon());
    }

    #[test]
    fn a_token_expiring_beyond_the_leeway_is_fresh() {
        let expires_at = Utc::now() + Duration::seconds(REFRESH_LEEWAY_SECS * 10);
        assert!(!credentials(Some(expires_at)).expires_soon());
    }

    #[test]
    fn an_expired_token_is_stale() {
        let expires_at = Utc::now() - Duration::seconds(1);
        assert!(credentials(Some(expires_at)).expires_soon());
    }
}
