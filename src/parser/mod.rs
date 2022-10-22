use std::str::FromStr;
use serde::Deserialize;
use crate::json_ext::JsonExt;
use crate::{AlbumReference, ArtistReference, PlaylistItem, Thumbnail};

pub fn parse_title(value: &serde_json::Value) -> Option<String> {
    let title = value.nav(&[&"title", &"runs", &0, &"text"])?
        .as_str()?
        .to_string();

    Some(title)
}

pub fn parse_playlist_items(value: &serde_json::Value) -> Vec<PlaylistItem> {
    value.as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .flat_map(parse_playlist_item)
        .collect()
}

fn parse_playlist_item(value: &serde_json::Value) -> Option<PlaylistItem> {
    let data = value.nav(&[&"musicResponsiveListItemRenderer"])?;
    let video_id = data.nav(&[
        &"overlay",
        &"musicItemThumbnailOverlayRenderer",
        &"content",
        &"musicPlayButtonRenderer",
        &"playNavigationEndpoint",
        &"watchEndpoint",
        &"videoId"
    ]).and_then(value_to_string)?;
    let title = data.nav(&[
        &"flexColumns",
        &0,
        &"musicResponsiveListItemFlexColumnRenderer",
        &"text",
        &"runs",
        &0,
        &"text"
    ]).and_then(value_to_string)?;
    let artists = parse_song_artists(&data);
    let album = parse_song_album(&data);
    let thumbnails = parse_playlist_item_thumbnail(&data);
    let is_explicit = data.nav(&[&"badges", &0, &"musicInlineBadgeRenderer", &"accessibilityData", &"accessibilityData", &"label"]).is_some();
    let duration = parse_duration(&data);
    let duration_seconds = duration.as_ref().and_then(|duration| {
        duration
            .split(":")
            .map(|part| u64::from_str(part).ok())
            .collect::<Option<Vec<_>>>()
    }).map(|parts| {
        parts
            .into_iter()
            .rev()
            .zip(&[1, 60, 3600])
            .map(|(part, multiplier)| part * *multiplier)
            .sum()
    });

    Some(PlaylistItem {
        video_id,
        title,
        thumbnails,
        artists,
        album,
        duration,
        duration_seconds,
        is_available: true,
        is_explicit,
    })
}

fn parse_duration(value: &serde_json::Value) -> Option<String> {
    let columns = value.nav(&[
        &"fixedColumns",
        &0,
        &"musicResponsiveListItemFixedColumnRenderer",
        &"text"
    ])?;

    columns
        .nav(&[&"simpleText"])
        .or_else(|| columns.nav(&[&"runs", &0, &"text"]))
        .and_then(value_to_string)
}

fn parse_song_artists(value: &serde_json::Value) -> Vec<ArtistReference> {
    let artist_runs = get_artist_runs(value).unwrap_or_default();

    if artist_runs.is_empty() {
        return Default::default();
    }

    let mut artists = Vec::new();

    for i in 0..((artist_runs.len() / 2) + 1) {
        let run = &artist_runs[i * 2];
        let artist = ArtistReference {
            id: run.nav(&[&"navigationEndpoint", &"browseEndpoint", &"browseId"]).and_then(value_to_string).unwrap_or_default(),
            name: run.nav(&[&"text"]).and_then(value_to_string).unwrap_or_default(),
        };

        artists.push(artist);
    }

    artists
}

fn get_artist_runs(value: &serde_json::Value) -> Option<Vec<serde_json::Value>> {
    let artist_runs = value.nav(&[
        &"flexColumns",
        &1,
        &"musicResponsiveListItemFlexColumnRenderer",
        &"text",
        &"runs",
    ])?.as_array()?.clone();

    Some(artist_runs)
}

fn parse_song_album(value: &serde_json::Value) -> Option<AlbumReference> {
    let album_item = get_flex_column_with_page_type(value, "MUSIC_PAGE_TYPE_ALBUM")?;

    Some(AlbumReference {
        id: album_item.navigation_endpoint.browse_endpoint.browse_id,
        name: album_item.text,
    })
}

fn parse_playlist_item_thumbnail(value: &serde_json::Value) -> Vec<Thumbnail> {
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

pub fn value_to_string(value: serde_json::Value) -> Option<String> {
    Some(value.as_str()?.to_string())
}

fn get_flex_column_with_page_type(value: &serde_json::Value, page_type: &str) -> Option<MusicResponsiveListItemFlexColumnTextRun> {
    let flex_columns = value.nav(&[&"flexColumns"])?;
    let flex_columns = flex_columns.as_array()?;

    flex_columns
        .iter()
        .flat_map(|column| column.nav(&[
            &"musicResponsiveListItemFlexColumnRenderer",
            &"text",
            &"runs",
            &0,
        ]))
        .find(|column| get_flex_column_page_type(column) == Some(page_type.to_string()))
        .and_then(|run| serde_json::from_value(run).ok())
}

fn get_flex_column_page_type(value: &serde_json::Value) -> Option<String> {
    let page_type = value.nav(&[
        &"navigationEndpoint",
        &"browseEndpoint",
        &"browseEndpointContextSupportedConfigs",
        &"browseEndpointContextMusicConfig",
        &"pageType"
    ])?;
    let page_type = page_type.as_str()?;

    Some(page_type.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MusicResponsiveListItemFlexColumnTextRun {
    text: String,
    navigation_endpoint: NavigationEndpoint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NavigationEndpoint {
    browse_endpoint: BrowseEndpoint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowseEndpoint {
    browse_id: String,
    #[serde(rename = "browseEndpointContextSupportedConfigs")]
    supported_configs: BrowseEndpointContextSupportedConfigs,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowseEndpointContextSupportedConfigs {
    #[serde(rename = "browseEndpointContextMusicConfig")]
    config: BrowseEndpointContextMusicConfig
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowseEndpointContextMusicConfig {
    page_type: String
}
