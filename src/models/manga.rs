use serde::{Deserialize, Serialize};

use super::Image;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    #[serde(rename = "comicId")]
    pub comic_id: String,
    pub slug: String,
    pub title: String,
    pub cover: Image,
    #[serde(rename = "noVolume")]
    pub no_volume: bool,
    pub genres: Vec<ComicTag>,
    pub metadata: ComicMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicMetadata {
    pub completed: Option<bool>,
    pub creators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicTag {
    #[serde(rename = "tagId")]
    id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEpisodes {
    #[serde(rename = "episodeNumber")]
    pub episode: i32,
    pub pages: Vec<Image>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contents {
    pub episodes: Vec<ContentEpisodes>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub slug: String,
    #[serde(rename = "volumeNumber")]
    pub number: i32,
    pub name: String,
    pub purchased: bool,
    #[serde(rename = "readerSkipCover")]
    pub reader_skip_cover: bool,
    pub cover: Image,
    #[serde(rename = "releasesAt")]
    pub release_at: Option<String>,
    pub price: Option<String>,
}
