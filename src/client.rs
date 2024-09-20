use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::{collections::HashMap, fmt::Debug, sync::LazyLock};

use crate::{
    config::{save_config, Config},
    kp::{self, RSAError},
    models::{Comic, ComicVolumes, Contents, GraphQLResponse},
};

const FF_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:130.0) Gecko/20100101 Firefox/130.0";
static BASE_HOST: LazyLock<String> = LazyLock::new(|| {
    let decoded = general_purpose::STANDARD
        .decode("ZW1hcWkuY29t")
        .expect("Failed to decode BASE_HOST");

    String::from_utf8(decoded).expect("Failed to convert BASE_HOST to String")
});
static BASE_URL: LazyLock<String> = LazyLock::new(|| {
    let decoded = general_purpose::STANDARD
        .decode("aHR0cHM6Ly9lbWFxaS5jb20=")
        .expect("Failed to decode BASE_URL");

    String::from_utf8(decoded).expect("Failed to convert BASE_URL to String")
});
static API_URL: LazyLock<String> = LazyLock::new(|| {
    let decoded = general_purpose::STANDARD
        .decode("aHR0cHM6Ly9hcGkuZW1hcWkuY29tL2dyYXBocWw=")
        .expect("Failed to decode API_URL");

    String::from_utf8(decoded).expect("Failed to convert API_URL to String")
});
static API_HOST: LazyLock<String> = LazyLock::new(|| {
    let decoded = general_purpose::STANDARD
        .decode("YXBpLmVtYXFpLmNvbQ==")
        .expect("Failed to decode API_HOST");

    String::from_utf8(decoded).expect("Failed to convert API_HOST to String")
});

static TOKEN_AUTH: LazyLock<String> = LazyLock::new(|| {
    let decoded = general_purpose::STANDARD
        .decode("QUl6YVN5QzZOYVE1dk9PYXJ0SUdUUEpIR2dTUDFPQmpwU05Lclpv")
        .expect("Failed to decode TOKEN_AUTH");

    String::from_utf8(decoded).expect("Failed to convert TOKEN_AUTH to String")
});

pub enum ClientError {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    TokenRefresh(String),
    RSA(RSAError),
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<RSAError> for ClientError {
    fn from(e: RSAError) -> Self {
        Self::RSA(e)
    }
}

pub struct Client {
    client: reqwest::Client,
    config: Config,
    priv_key: rsa::RsaPrivateKey,
    pub_key: rsa::RsaPublicKey,
}

impl Client {
    pub fn new(config: &mut Config) -> Result<Self, ClientError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(FF_UA),
        );
        headers.insert(
            reqwest::header::HOST,
            reqwest::header::HeaderValue::from_static(&API_HOST),
        );

        let client = reqwest::Client::builder()
            .http2_adaptive_window(true)
            .default_headers(headers)
            .build()?;

        let (priv_key, pub_key) = if !config.has_key() {
            config.generate_key_pair()?
        } else {
            config.get_key_pair()?
        };

        Ok(Self {
            client,
            config: config.clone(),
            priv_key,
            pub_key,
        })
    }

    /// Refresh the token of the client.
    ///
    /// The following function will be called on each request to ensure the token is always valid.
    ///
    /// The first request will always be a token refresh, and subsequent requests will only refresh
    /// if the token is expired.
    pub async fn refresh_token(&mut self) -> Result<(), ClientError> {
        // If the expiry time is set and it's not expired, return early
        if !self.config.is_expired() {
            return Ok(());
        }

        let json_data = json!({
            "grantType": "refresh_token",
            "refreshToken": self.config.refresh_token(),
        });

        let client = reqwest::Client::builder()
            .http2_adaptive_window(true)
            .build()?;
        let request = client
            .post("https://securetoken.googleapis.com/v1/token")
            .header(reqwest::header::USER_AGENT, FF_UA)
            .query(&[("key", TOKEN_AUTH.to_string())])
            .json(&json_data)
            .send()
            .await?;

        let response = request
            .json::<crate::config::google_auth::SecureTokenResponse>()
            .await?;

        self.config.set_access_token(&response.access_token);
        self.config.set_refresh_token(&response.refresh_token);
        self.config.set_expires_at(response.expires_at());

        save_config(&self.config);

        Ok(())
    }

    async fn query_protected<T>(
        &mut self,
        query: impl Into<String>,
        variables: HashMap<String, serde_json::Value>,
    ) -> Result<GraphQLResponse<T>, ClientError>
    where
        T: serde::de::DeserializeOwned + Debug + Clone,
    {
        let json_data = json!({
            "query": query.into(),
            "variables": variables,
        });

        let x_hash = kp::create_xhash(&self.pub_key)?;

        let req = self
            .client
            .post(&*API_URL)
            .header(reqwest::header::USER_AGENT, FF_UA)
            .header(reqwest::header::HOST, API_HOST.to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("x-hash", x_hash)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.config.access_token()),
            )
            .json(&json_data)
            .send()
            .await?;

        let response = req.json::<GraphQLResponse<T>>().await?;

        Ok(response)
    }

    async fn query<T>(
        &mut self,
        query: impl Into<String>,
        variables: HashMap<String, String>,
    ) -> Result<GraphQLResponse<T>, ClientError>
    where
        T: serde::de::DeserializeOwned + Debug + Clone,
    {
        let json_data = json!({
            "query": query.into(),
            "variables": variables,
        });

        let req = self
            .client
            .post(&*API_URL)
            .header(reqwest::header::USER_AGENT, FF_UA)
            .header(reqwest::header::HOST, API_HOST.to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.config.access_token()),
            )
            .json(&json_data)
            .send()
            .await?;

        let response = req.json::<GraphQLResponse<T>>().await?;

        Ok(response)
    }

    pub async fn search(&mut self, query: impl Into<String>) -> Result<Vec<Comic>, ClientError> {
        self.refresh_token().await?;

        let query = r#"query searchManga($query:String!) {
            search(input:{keyword:$query})  {
                comicId
                slug
                title
                cover {
                    url
                    height
                }
                noVolume
            }
        }"#;

        let q_s: String = query.into();

        let variables: HashMap<String, String> = [("query", q_s)]
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        let response = self
            .query::<crate::models::SearchQuery>(query, variables)
            .await?;

        Ok(response.data.search)
    }

    pub async fn get_volumes(
        &mut self,
        slug: impl Into<String>,
    ) -> Result<ComicVolumes, ClientError> {
        self.refresh_token().await?;

        let query = r#"query getVolumes($slug:String!) {
            comicVolumes(comicSlug:$slug) {
                comic {
                    comicId
                    slug
                    title
                    cover {
                        url
                        height
                    }
                    noVolume
                }
                volumes {
                    slug
                    volumeNumber
                    name
                    purchased
                    readerSkipCover
                    cover {
                        height
                        url
                    }
                }
            }
        }"#;

        let slug_s: String = slug.into();

        let variables: HashMap<String, String> = [("slug", slug_s)]
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        let response = self
            .query::<crate::models::VolumesQuery>(query, variables)
            .await?;

        Ok(response.data.comic_volumes)
    }

    pub async fn get_contents(
        &mut self,
        id: impl Into<String>,
        volume: i32,
    ) -> Result<Contents, ClientError> {
        self.refresh_token().await?;

        let query = r#"query getMangaContents($comicId: String!, $volumeNumber: Int!) {
            manga(comicId:$comicId, volumeNumber:$volumeNumber) {
                contents {
                    episodes {
                        episodeNumber
                        pages {
                            height
                            url
                        }
                    }
                    hash
                }
            }
        }"#;

        let comic_id: String = id.into();

        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        variables.insert("comicId".to_string(), serde_json::Value::String(comic_id));
        variables.insert(
            "volumeNumber".to_string(),
            serde_json::Value::Number(volume.into()),
        );

        let response = self
            .query_protected::<crate::models::ComicContentsQuery>(query, variables)
            .await?;

        Ok(response.data.manga.contents)
    }
}
