use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

pub mod common;
pub mod manga;
pub mod users;

pub use common::*;
pub use manga::*;
pub use users::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse<T> {
    #[serde(bound(
        deserialize = "T: DeserializeOwned + Clone + Debug",
        serialize = "T: Serialize + Clone + Debug"
    ))]
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub search: Vec<Comic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicVolumes {
    pub comic: Comic,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumesQuery {
    #[serde(rename = "comicVolumes")]
    pub comic_volumes: ComicVolumes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicContents {
    pub contents: Contents,
    pub volume: Volume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicContentsQuery {
    pub manga: ComicContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoQuery {
    pub user: User,
    #[serde(rename = "userProfile")]
    pub profile: UserProfile,
}
