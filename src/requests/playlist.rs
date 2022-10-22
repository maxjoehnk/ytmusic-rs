use crate::json_ext::JsonExt;
use crate::{PlaylistItem, YoutubeMusicClient, YoutubeMusicError};
use crate::parser::{parse_playlist_items, parse_title, value_to_string};

const BROWSE_ENDPOINT: &str = "browse";

impl YoutubeMusicClient {
    pub async fn get_playlist(&self, id: &str) -> Result<Option<Playlist>, YoutubeMusicError> {
        let id = if id.starts_with("VL") {
            id.to_string()
        }else {
            "VL".to_owned() + id
        };
        let res: serde_json::Value = self.get(BROWSE_ENDPOINT, &serde_json::json!({
            "browse_id": id
        })).await?;

        let playlist = parse_playlist(&res);

        Ok(playlist)
    }
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub tracks: Vec<PlaylistItem>,
}

fn parse_playlist(response: &serde_json::Value) -> Option<Playlist> {
    let results = response.nav(&[
            &"contents",
            &"singleColumnBrowseResultsRenderer",
            &"tabs",
            &0,
            &"tabRenderer",
            &"content",
            &"sectionListRenderer",
            &"contents",
            &0,
            &"musicPlaylistShelfRenderer",
        ])?;
    let id = results.nav(&[&"playlistId"]).and_then(value_to_string)?;
    let header = response.nav(&[&"header", &"musicEditablePlaylistDetailHeaderRenderer"])
        .or_else(|| Some(response.clone()))
        .and_then(|header| header.nav(&[&"header", &"musicDetailHeaderRenderer"]))?;
    let title = parse_title(&header)?;
    let tracks = parse_playlist_items(&results.nav(&[&"contents"])?);

    let playlist = Playlist {
        id,
        title,
        tracks,
    };

    Some(playlist)
}
