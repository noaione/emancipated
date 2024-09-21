use std::{io::Write, path::PathBuf};

use directories::BaseDirs;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{ser, Deserialize, Serialize};

use crate::{
    kp::{self, hash_b64},
    term::ConsoleChoice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    email: String,
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    /// Path to public key
    public_key: String,
    /// Path to private key
    private_key: String,
}

impl Config {
    pub fn is_expired(&self) -> bool {
        let unix_time = time::OffsetDateTime::now_utc().unix_timestamp();
        self.expires_at < unix_time
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn set_access_token(&mut self, access_token: impl Into<String>) {
        self.access_token = access_token.into();
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn set_refresh_token(&mut self, refresh_token: impl Into<String>) {
        self.refresh_token = refresh_token.into();
    }

    pub fn expires_at(&self) -> i64 {
        self.expires_at
    }

    pub fn set_expires_at(&mut self, expires_at: i64) {
        self.expires_at = expires_at;
    }

    pub fn public_key(&self) -> PathBuf {
        PathBuf::from(&self.public_key)
    }

    pub fn private_key(&self) -> PathBuf {
        PathBuf::from(&self.private_key)
    }

    pub fn get_key_pair(&self) -> Result<(RsaPrivateKey, RsaPublicKey), kp::RSAError> {
        kp::load_key_pair(&self.private_key(), &self.public_key())
    }

    pub fn set_public_key(&mut self, public_key: &str) {
        self.public_key = public_key.to_string();
    }

    pub fn set_private_key(&mut self, private_key: &str) {
        self.private_key = private_key.to_string();
    }

    pub fn has_key(&self) -> bool {
        !self.public_key.is_empty() && !self.private_key.is_empty()
    }

    pub fn generate_key_pair(&mut self) -> Result<(RsaPrivateKey, RsaPublicKey), kp::RSAError> {
        let write_dir = get_user_path();
        let (private_key, public_key) = kp::generate_key_pair()?;

        let email_64 = hash_b64(&self.email);

        let private_key_path = write_dir.join(format!("{}_kp.pem", email_64));
        let public_key_path = write_dir.join(format!("{}_kp.pub", email_64));

        kp::write_key_pair(
            &private_key_path,
            &public_key_path,
            &private_key,
            &public_key,
        )?;

        self.set_private_key(private_key_path.to_str().unwrap());
        self.set_public_key(public_key_path.to_str().unwrap());

        Ok((private_key, public_key))
    }
}

impl From<google_auth::VerifyPasswordResponseMinimal> for Config {
    fn from(value: google_auth::VerifyPasswordResponseMinimal) -> Self {
        Self {
            email: value.email.clone(),
            access_token: value.id_token.clone(),
            refresh_token: value.refresh_token.clone(),
            expires_at: value.expires_at(),
            private_key: "".to_string(),
            public_key: "".to_string(),
        }
    }
}

impl From<&google_auth::VerifyPasswordResponseMinimal> for Config {
    fn from(value: &google_auth::VerifyPasswordResponseMinimal) -> Self {
        Self {
            email: value.email.clone(),
            access_token: value.id_token.clone(),
            refresh_token: value.refresh_token.clone(),
            expires_at: value.expires_at(),
            private_key: "".to_string(),
            public_key: "".to_string(),
        }
    }
}

pub(crate) fn get_user_path() -> std::path::PathBuf {
    #[cfg(windows)]
    let user_path = {
        let mut local_appdata: std::path::PathBuf =
            BaseDirs::new().unwrap().config_local_dir().to_path_buf();
        local_appdata.push("EmancipatedRs");
        local_appdata
    };
    #[cfg(not(windows))]
    let user_path: std::path::PathBuf = {
        let mut home = BaseDirs::new().unwrap().home_dir().to_path_buf();
        home.push(".emancipatedrs");
        home
    };
    user_path
}

pub(crate) fn find_any_config() -> Vec<Config> {
    let user_path = get_user_path();
    let mut configs = vec![];

    if user_path.exists() {
        // glob dir for config_*.json
        let read_dir = std::fs::read_dir(user_path).unwrap();
        for file in read_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.starts_with("config_") && file_name.ends_with(".json") {
                    let file = std::fs::File::open(path).unwrap();
                    let config: Config = serde_json::from_reader(file).unwrap();
                    configs.push(config);
                }
            }
        }
    }

    configs
}

pub(crate) fn get_config(email: impl Into<String>) -> Option<Config> {
    let email = email.into();
    let user_path = get_user_path();
    let email_64 = hash_b64(&email);
    let file_path = user_path.join(format!("config_{}.json", email_64));

    if file_path.exists() {
        let file = std::fs::File::open(file_path).unwrap();
        let config: Config = serde_json::from_reader(file).unwrap();
        return Some(config);
    }

    None
}

pub(crate) fn select_single_account(
    email: Option<&str>,
    term: &crate::term::Terminal,
) -> Option<Config> {
    if let Some(email) = email {
        let config = get_config(email);

        if let Some(config) = config {
            return Some(config.clone());
        }

        term.warn(&format!("Account ID {} not found!", email));

        return None;
    }

    let all_configs = find_any_config();
    let all_choices: Vec<ConsoleChoice> = all_configs
        .iter()
        .map(|c| ConsoleChoice {
            name: c.email().to_string(),
            value: c.email().to_string(),
        })
        .collect();

    if all_configs.is_empty() {
        term.warn("No accounts found!");
        return None;
    }

    // only 1? return
    if all_configs.len() == 1 {
        return Some(all_configs[0].clone());
    }

    let selected = term.choice("Select an account:", all_choices);
    match selected {
        Some(selected) => {
            let config = all_configs
                .iter()
                .find(|c| c.email() == selected.name)
                .unwrap()
                .clone();

            Some(config)
        }
        None => None,
    }
}

pub(crate) fn save_config(config: &Config) {
    let user_path = get_user_path();
    if !user_path.exists() {
        std::fs::create_dir_all(&user_path).unwrap();
    }

    let email_64 = hash_b64(&config.email);
    let file_path = user_path.join(format!("config_{}.json", email_64));
    let mut file = std::fs::File::create(file_path).unwrap();

    let results = serde_json::to_string_pretty(config).unwrap();

    file.write_all(results.as_bytes()).unwrap();
}

pub(crate) mod google_auth {
    use serde::{Deserialize, Serialize};

    /// Object representing the response of the auth request.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VerifyPasswordResponseMinimal {
        #[serde(rename = "idToken")]
        pub(crate) id_token: String,
        #[serde(rename = "expiresIn")]
        pub(crate) expires_in: String,
        #[serde(rename = "refreshToken")]
        pub(crate) refresh_token: String,
        pub(crate) email: String,
    }

    impl VerifyPasswordResponseMinimal {
        pub fn expires_at(&self) -> i64 {
            let expires_in: i64 = self.expires_in.parse().unwrap();
            let unix_time = time::OffsetDateTime::now_utc().unix_timestamp();

            unix_time + expires_in
        }
    }

    /// Object representing the response of the token exchange.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SecureTokenResponse {
        pub access_token: String,
        pub expires_in: String,
        pub token_type: String,
        pub refresh_token: String,
        pub id_token: String,
        pub user_id: String,
        pub project_id: String,
    }

    impl SecureTokenResponse {
        pub fn expires_at(&self) -> i64 {
            let expires_in: i64 = self.expires_in.parse().unwrap();
            let unix_time = time::OffsetDateTime::now_utc().unix_timestamp();

            unix_time + expires_in
        }
    }
}
