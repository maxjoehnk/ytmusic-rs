use std::time::SystemTime;
use serde::Deserialize;

use crate::{AlbumType, ArtistReference, PlaylistItem, Thumbnail, YoutubeMusicClient, YoutubeMusicError};
use crate::json_ext::JsonExt;
use crate::parser::{parse_playlist_items, parse_title, value_to_string};

const DAY_IN_SECONDS: u64 = 60 * 60 * 24;

const BROWSE_ENDPOINT: &str = "browse";
const GET_SONG_ENDPOINT: &str = "player";

impl YoutubeMusicClient {
    pub async fn get_album(&self, id: &str) -> Result<Option<BrowseAlbum>, YoutubeMusicError> {
        let res: serde_json::Value = self.get(BROWSE_ENDPOINT, &serde_json::json!({
            "browseId": id
        })).await?;

        let mut album = parse_album_header(&res);
        if let Some(album) = album.as_mut() {
            album.id = id.to_string();
            if let Some(contents) = res.nav(&[
                &"contents",
                &"singleColumnBrowseResultsRenderer",
                &"tabs",
                &0,
                &"tabRenderer",
                &"content",
                &"sectionListRenderer",
                &"contents",
                &0,
                &"musicShelfRenderer",
                &"contents",
            ]) {
                album.tracks = parse_playlist_items(&contents);
            }
        }

        Ok(album)
    }

    pub async fn get_song(&self, id: &str) -> Result<Option<BrowseSong>, YoutubeMusicError> {
        let signature_timestamp = Self::get_signature_timestamp();
        let res: BrowseSong = self.get(GET_SONG_ENDPOINT, &serde_json::json!({
            "playbackContext": {
                "contentPlaybackContext": {
                    "signatureTimestamp": signature_timestamp
                }
            },
            "video_id": id
        })).await?;

        Ok(Some(res))
    }

    pub async fn get_artist(&self, id: &str) -> Result<Option<BrowseArtist>, YoutubeMusicError> {
        let browse_id = if id.starts_with("MPLA") {
            &id[4..]
        }else {
            &id
        };
        let res: serde_json::Value = self.get(BROWSE_ENDPOINT, &serde_json::json!({
            "browseId": browse_id
        })).await?;

        let artist = parse_artist(id, res);

        Ok(artist)
    }

    fn get_signature_timestamp() -> u64 {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let secs = duration.as_secs();
        let days = secs / DAY_IN_SECONDS;

        days - 1
    }
}

fn parse_artist(id: &str, res: serde_json::Value) -> Option<BrowseArtist> {
    let contents = res.nav(&[
        &"contents",
        &"singleColumnBrowseResultsRenderer",
        &"tabs",
        &0,
        &"tabRenderer",
        &"content",
        &"sectionListRenderer",
        &"contents",
    ])?;
    let header = res.nav(&[
        &"header",
        &"musicImmersiveHeaderRenderer"
    ])?;
    let name = header.nav(&[&"title", &"runs", &0, &"text"])?
        .as_str()?
        .to_string();
    let thumbnails = parse_thumbnails(&header);

    Some(BrowseArtist {
        id: id.to_string(),
        name,
        thumbnails,
    })
}

#[derive(Debug, Clone)]
pub struct BrowseAlbum {
    pub id: String,
    pub title: String,
    pub album_type: AlbumType,
    pub thumbnails: Vec<Thumbnail>,
    pub description: Option<String>,
    pub artists: Vec<ArtistReference>,
    pub year: Option<String>,
    pub tracks: Vec<PlaylistItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseSong {
    pub video_details: VideoDetails
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub video_id: String,
    pub allow_ratings: bool,
    pub author: String,
    pub title: String,
    pub channel_id: String,
    pub is_crawlable: bool,
    pub is_live_content: bool,
    pub is_owner_viewing: bool,
    pub is_private: bool,
    pub is_unplugged_corpus: bool,
    pub length_seconds: String,
    pub thumbnail: Thumbnails,
}

#[derive(Debug, Clone)]
pub struct BrowseArtist {
    pub id: String,
    pub name: String,
    pub thumbnails: Vec<Thumbnail>
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Thumbnails {
    pub thumbnails: Vec<Thumbnail>
}

fn parse_album_header(value: &serde_json::Value) -> Option<BrowseAlbum> {
    let header = value.nav(&[
        &"header",
        &"musicDetailHeaderRenderer"
    ])?;
    let title = parse_title(&header)?;
    let album_type = header.nav(&[
        &"subtitle",
        &"runs",
        &0,
        &"text"
    ]).and_then(|album_type| serde_json::from_value(album_type).ok())?;
    let thumbnails = parse_thumbnail_cropped(&header);

    let description = header.nav(&[&"description", &"runs", &0, &"text"])
        .and_then(value_to_string);

    Some(BrowseAlbum {
        id: Default::default(),
        title,
        album_type,
        thumbnails,
        year: None,
        description,
        artists: Default::default(),
        tracks: Default::default(),
    })
}

fn parse_thumbnails(value: &serde_json::Value) -> Vec<Thumbnail> {
    value.nav(&[
        &"thumbnail",
        &"musicThumbnailRenderer",
        &"thumbnail",
        &"thumbnails"
    ])
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|thumbnail| serde_json::from_value(thumbnail).ok())
        .collect()
}

fn parse_thumbnail_cropped(value: &serde_json::Value) -> Vec<Thumbnail> {
    value.nav(&[
        &"thumbnail",
        &"croppedSquareThumbnailRenderer",
        &"thumbnail",
        &"thumbnails"
    ])
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|thumbnail| serde_json::from_value(thumbnail).ok())
        .collect()
}
