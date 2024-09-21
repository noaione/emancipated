use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::{collections::HashMap, fmt::Debug, sync::LazyLock};

use crate::{
    config::{google_auth::VerifyPasswordResponseMinimal, save_config, Config},
    image::ImageError,
    kp::{self, RSAError},
    models::{Comic, ComicVolumes, Contents, GraphQLResponse, GraphQLResponseError, UserInfoQuery},
};

const FF_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:130.0) Gecko/20100101 Firefox/130.0";
pub(crate) static BASE_HOST: LazyLock<String> = LazyLock::new(|| {
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

/// Error type that happens when parsing the response from the API
///
/// This is specifically for [`serde`] errors.
///
/// When formatted as a string, it will show the error message, status code, headers, and a JSON excerpt.
pub struct DetailedSerdeError {
    inner: serde_json::Error,
    status_code: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    url: reqwest::Url,
    raw_text: String,
}

impl DetailedSerdeError {
    /// Create a new instance of the error
    pub(crate) fn new(
        inner: serde_json::Error,
        status_code: reqwest::StatusCode,
        headers: &reqwest::header::HeaderMap,
        url: &reqwest::Url,
        raw_text: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            status_code,
            headers: headers.clone(),
            url: url.clone(),
            raw_text: raw_text.into(),
        }
    }

    /// Get the JSON excerpt from the raw text
    ///
    /// This will return a string that contains where the deserialization error happened.
    ///
    /// It will take 25 characters before and after the error position.
    pub fn get_json_excerpt(&self) -> String {
        let row_line = self.inner.line() - 1;
        let split_lines = self.raw_text.split('\n').collect::<Vec<&str>>();

        let position = self.inner.column();
        let start_idx = position.saturating_sub(25);
        let end_idx = position.saturating_add(25);

        // Bound the start and end index
        let start_idx = start_idx.max(0);
        let end_idx = end_idx.min(split_lines[row_line].len());

        split_lines[row_line][start_idx..end_idx].to_string()
    }
}

pub enum ClientError {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    DetailedSerde(DetailedSerdeError),
    TokenRefresh(String),
    RSA(RSAError),
    Image(ImageError),
    GraphQLError(GraphQLResponseError),
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

impl From<ImageError> for ClientError {
    fn from(e: ImageError) -> Self {
        Self::Image(e)
    }
}

impl From<DetailedSerdeError> for ClientError {
    fn from(e: DetailedSerdeError) -> Self {
        Self::DetailedSerde(e)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "Reqwest Error: {}", e),
            Self::Serde(e) => write!(f, "Serde Error: {}", e),
            Self::TokenRefresh(e) => write!(f, "Token Refresh Error: {}", e),
            Self::RSA(e) => write!(f, "RSA Error: {}", e),
            Self::Image(e) => write!(f, "Image Error: {}", e),
            Self::GraphQLError(e) => write!(f, "GraphQL Error: {}", e),
            Self::DetailedSerde(e) => write!(
                f,
                "Serde Error: {}\nStatus Code: {}\nHeaders: {:?}\nURL: {}\nJSON excerpt: {}",
                e.inner,
                e.status_code,
                e.headers,
                e.url,
                e.get_json_excerpt()
            ),
        }
    }
}

impl std::fmt::Debug for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "Reqwest Error: {}", e),
            Self::Serde(e) => write!(f, "Serde Error: {}", e),
            Self::TokenRefresh(e) => write!(f, "Token Refresh Error: {}", e),
            Self::RSA(e) => write!(f, "RSA Error: {}", e),
            Self::Image(e) => write!(f, "Image Error: {}", e),
            Self::GraphQLError(e) => write!(f, "GraphQL Error: {}", e),
            Self::DetailedSerde(e) => write!(
                f,
                "Serde Error: {}\nStatus Code: {}\nHeaders: {:?}\nURL: {}\nJSON excerpt: {}",
                e.inner,
                e.status_code,
                e.headers,
                e.url,
                e.get_json_excerpt()
            ),
        }
    }
}

pub struct Client {
    client: reqwest::Client,
    config: Config,
    priv_key: rsa::RsaPrivateKey,
    pub_key: rsa::RsaPublicKey,
}

impl Client {
    pub fn new(config: &mut Config, proxy: Option<reqwest::Proxy>) -> Result<Self, ClientError> {
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
            .default_headers(headers);

        let client = match proxy {
            Some(proxy) => client.proxy(proxy),
            None => client,
        }
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

    /// Get the configuration of the client.
    pub fn get_config_owned(&self) -> Config {
        self.config.clone()
    }

    pub fn get_config(&self) -> &Config {
        &self.config
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
        self.refresh_token().await?;

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
            .await?
            // since all graphql requests will be 200 OK, we need to check the response
            .error_for_status()?;

        let status_code = req.status();
        let headers = req.headers().clone();
        let url = req.url().clone();
        let text_data = req.text().await?;

        // Check for errors
        match serde_json::from_str::<GraphQLResponseError>(&text_data) {
            Ok(error_response) => {
                return Err(ClientError::GraphQLError(error_response));
            }
            Err(_) => {}
        }

        // Actual response
        let resp = serde_json::from_str::<GraphQLResponse<T>>(&text_data)
            .map_err(|e| DetailedSerdeError::new(e, status_code, &headers, &url, text_data))?;

        Ok(resp)
    }

    async fn query<T>(
        &mut self,
        query: impl Into<String>,
        variables: HashMap<String, String>,
    ) -> Result<GraphQLResponse<T>, ClientError>
    where
        T: serde::de::DeserializeOwned + Debug + Clone,
    {
        self.refresh_token().await?;

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
            .await?
            // since all graphql requests will be 200 OK, we need to check the response
            .error_for_status()?;

        let status_code = req.status();
        let headers = req.headers().clone();
        let url = req.url().clone();
        let text_data = req.text().await?;

        // Check for errors
        match serde_json::from_str::<GraphQLResponseError>(&text_data) {
            Ok(error_response) => {
                return Err(ClientError::GraphQLError(error_response));
            }
            Err(_) => {}
        }

        // Actual response
        let resp = serde_json::from_str::<GraphQLResponse<T>>(&text_data)
            .map_err(|e| DetailedSerdeError::new(e, status_code, &headers, &url, text_data))?;

        Ok(resp)
    }

    pub async fn search(&mut self, keyword: impl Into<String>) -> Result<Vec<Comic>, ClientError> {
        let query = r#"query searchManga($query:String!) {
            search(input:{keyword:$query})  {
                comicId
                slug
                title
                cover {
                    url
                    height
                }
                genres {
                    name
                    tagId
                }
                metadata {
                    completed
                    creators
                }
                noVolume
            }
        }"#;

        let q_s: String = keyword.into();

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
                    genres {
                        name
                        tagId
                    }
                    metadata {
                        completed
                        creators
                    }
                }
                volumes {
                    slug
                    volumeNumber
                    name
                    purchased
                    readerSkipCover
                    releasesAt
                    price
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

    pub async fn get_user(&mut self) -> Result<UserInfoQuery, ClientError> {
        let query = r#"query getUserInfo {
            user {
                coins
                userId
                deletedAt
            }
            userProfile {
                userId
                pronouns
                dateOfBirth
            }
        }"#;

        let response = self.query::<UserInfoQuery>(query, HashMap::new()).await?;

        Ok(response.data)
    }

    pub async fn login(
        email: impl Into<String>,
        password: impl Into<String>,
        proxy: Option<reqwest::Proxy>,
    ) -> Result<VerifyPasswordResponseMinimal, ClientError> {
        let email_s: String = email.into();
        let password_s: String = password.into();

        let json_data = json!({
            "email": email_s,
            "password": password_s,
            "returnSecureToken": true,
            "clientType": "CLIENT_TYPE_WEB",
        });

        let client = reqwest::Client::builder().http2_adaptive_window(true);
        let client = match proxy {
            Some(proxy) => client.proxy(proxy),
            None => client,
        }
        .build()?;
        let request = client
            .post("https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword")
            .header(reqwest::header::USER_AGENT, FF_UA)
            .query(&[("key", TOKEN_AUTH.to_string())])
            .json(&json_data)
            .send()
            .await?;

        let response = request.json::<VerifyPasswordResponseMinimal>().await?;

        Ok(response)
    }
}
