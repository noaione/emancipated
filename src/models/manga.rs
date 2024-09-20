use serde::{Deserialize, Serialize};

use super::Image;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    #[serde(rename = "comicId")]
    comic_id: String,
    slug: String,
    title: String,
    cover: Image,
    #[serde(rename = "noVolume")]
    no_volume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEpisodes {
    #[serde(rename = "episodeNumber")]
    episode: i32,
    pages: Vec<Image>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contents {
    episodes: Vec<ContentEpisodes>,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    slug: String,
    #[serde(rename = "volumeNumber")]
    number: i32,
    name: String,
    purchased: bool,
    #[serde(rename = "readerSkipCover")]
    reader_skip_cover: bool,
    cover: Image,
}
