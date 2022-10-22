use serde::{Deserialize, Serialize};

use crate::{AlbumType, ArtistReference, Thumbnail, YoutubeMusicClient, YoutubeMusicError};
use crate::json_ext::JsonExt;
use crate::parser::parse_title;

const LIBRARY_ENDPOINT: &str = "browse";

impl YoutubeMusicClient {
    pub async fn get_library_playlists(&self, limit: Option<usize>) -> Result<Vec<LibraryPlaylist>, YoutubeMusicError> {
        let res: serde_json::Value = self.get(LIBRARY_ENDPOINT, &LibraryRequest {
            browse_id: BrowseId::Playlists
        }).await?;

        let contents = get_library_contents(res).ok_or(YoutubeMusicError::ApiError)?;
        let contents: Vec<SectionListItem> = serde_json::from_value(contents)?;
        let playlists = contents.into_iter()
            .filter_map(|content| match content {
                SectionListItem::ItemSectionRenderer { contents } => Some(contents),
                _ => None,
            })
            .flatten()
            .filter_map(|item_section| {
                if let ItemSectionRenderer::GridRenderer { items } = item_section {
                    Some(items)
                }else {
                    None
                }
            })
            .flatten()
            .skip(1)
            .flat_map(parse_playlist)
            .collect::<Vec<_>>();

        Ok(playlists)
    }

    pub async fn get_library_songs(&self, limit: Option<usize>) -> Result<Vec<LibrarySong>, YoutubeMusicError> {
        todo!()
    }

    pub async fn get_library_albums(&self, limit: Option<usize>) -> Result<Vec<LibraryAlbum>, YoutubeMusicError> {
        let res: serde_json::Value = self.get(LIBRARY_ENDPOINT, &LibraryRequest {
            browse_id: BrowseId::Albums
        }).await?;

        let contents = get_library_contents(res).ok_or(YoutubeMusicError::ApiError)?;
        let contents: Vec<SectionListItem> = serde_json::from_value(contents)?;
        let albums = contents.into_iter()
            .filter_map(|content| match content {
                SectionListItem::ItemSectionRenderer { contents } => Some(contents),
                _ => None,
            })
            .flatten()
            .filter_map(|item_section| {
                if let ItemSectionRenderer::GridRenderer { items } = item_section {
                    Some(items)
                }else {
                    None
                }
            })
            .flatten()
            .flat_map(parse_album)
            .collect::<Vec<_>>();

        Ok(albums)
    }

    pub async fn get_library_artists(&self, limit: Option<usize>) -> Result<Vec<LibraryArtist>, YoutubeMusicError> {
        let res: serde_json::Value = self.get(LIBRARY_ENDPOINT, &LibraryRequest {
            browse_id: BrowseId::TrackArtists
        }).await?;

        let contents = get_library_contents(res).ok_or(YoutubeMusicError::ApiError)?;
        let contents: Vec<SectionListItem> = serde_json::from_value(contents)?;
        let artists = contents.into_iter()
            .filter_map(|content| match content {
                SectionListItem::ItemSectionRenderer { contents } => Some(contents),
                _ => None,
            })
            .flatten()
            .filter_map(|item_section| {
                if let ItemSectionRenderer::MusicShelfRenderer { contents: items } = item_section {
                    Some(items)
                }else {
                    None
                }
            })
            .flatten()
            .flat_map(parse_artist)
            .collect::<Vec<_>>();

        Ok(artists)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryRequest {
    browse_id: BrowseId,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum BrowseId {
    #[serde(rename = "FEmusic_liked_playlists")]
    Playlists,
    #[serde(rename = "FEmusic_liked_videos")]
    Songs,
    #[serde(rename = "FEmusic_liked_albums")]
    Albums,
    #[serde(rename = "FEmusic_library_corpus_track_artists")]
    TrackArtists,
}

#[derive(Debug, Clone)]
pub struct LibraryPlaylist {
    pub id: String,
    pub title: String,
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Clone)]
pub struct LibrarySong {}

#[derive(Debug, Clone)]
pub struct LibraryAlbum {
    pub id: String,
    pub playlist_id: Option<String>,
    pub title: String,
    pub year: Option<String>,
    pub album_type: Option<AlbumType>,
    pub artists: Vec<ArtistReference>,
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Clone)]
pub struct LibraryArtist {
    pub id: String,
    pub artist: String,
    pub thumbnails: Vec<Thumbnail>,
}

fn get_library_contents(value: serde_json::Value) -> Option<serde_json::Value> {
    value.nav(&[
        &"contents",
        &"singleColumnBrowseResultsRenderer",
        &"tabs",
        &0,
        &"tabRenderer",
        &"content",
        &"sectionListRenderer",
        &"contents"
    ])
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SectionListItem {
    ItemSectionRenderer {
        contents: Vec<ItemSectionRenderer>,
    },
    MusicCarouselShelfRenderer {},
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ItemSectionRenderer {
    GridRenderer {
        items: Vec<serde_json::Value>
    },
    MusicShelfRenderer {
        contents: Vec<serde_json::Value>
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Item {
    MusicTwoRowItemRenderer(serde_json::Value)
}

fn parse_playlist(value: serde_json::Value) -> Option<LibraryPlaylist> {
    let value = value.nav(&[&"musicTwoRowItemRenderer"])?;
    let title = parse_title(&value)?;
    let id = value.nav(&[&"title", &"runs", &0, &"navigationEndpoint", &"browseEndpoint", &"browseId"])?
        .as_str()?[2..]
        .to_string();
    let thumbnails = parse_thumbnails(&value);

    Some(LibraryPlaylist {
        id,
        title,
        thumbnails,
    })
}

fn parse_album(value: serde_json::Value) -> Option<LibraryAlbum> {
    let value = value.nav(&[&"musicTwoRowItemRenderer"])?;
    let title = parse_title(&value)?;
    let id = value.nav(&[&"title", &"runs", &0, &"navigationEndpoint", &"browseEndpoint", &"browseId"])?
        .as_str()?
        .to_string();
    let playlist_id = get_playlist_id(&value);
    let mut year = None;
    let mut album_type = None;
    let artists = Vec::new();
    let thumbnails = parse_thumbnails(&value);

    if let Some(runs) = value.nav(&[&"subtitle", &"runs"]) {
        let run_count = runs.as_array()?.len();
        if run_count == 1 {
            year = runs.nav(&[&0, &"text"]).and_then(|text| text.as_str().map(|t| t.to_string()));
        }else {
            album_type = runs.nav(&[&0, &"text"]).and_then(|album_type| serde_json::from_value(album_type).ok());
        }
    }

    Some(LibraryAlbum {
        id,
        playlist_id,
        title,
        year,
        album_type,
        artists,
        thumbnails,
    })
}

fn parse_artist(value: serde_json::Value) -> Option<LibraryArtist> {
    let value = value.nav(&[&"musicResponsiveListItemRenderer"])?;
    let title = value.nav(&[
        &"flexColumns",
        &0,
        &"musicResponsiveListItemFlexColumnRenderer",
        &"text",
        &"runs",
        &0,
        &"text"
    ])?
        .as_str()?
        .to_string();
    let id = value.nav(&[&"navigationEndpoint", &"browseEndpoint", &"browseId"])?
        .as_str()?
        .to_string();
    let thumbnails = parse_thumbnails(&value);

    Some(LibraryArtist {
        id,
        artist: title,
        thumbnails,
    })
}

fn parse_thumbnails(value: &serde_json::Value) -> Vec<Thumbnail> {
    value.nav(&[
        &"thumbnailRenderer",
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

fn get_playlist_id(value: &serde_json::Value) -> Option<String> {
    let playlist_id = value.nav(&[
        &"menu",
        &"menuRenderer",
        &"items",
        &0,
        &"menuNavigationItemRenderer",
        &"navigationEndpoint",
        &"watchPlaylistEndpoint",
        &"playlistId"
    ])?
        .as_str()?[2..]
        .to_string();

    Some(playlist_id)
}
