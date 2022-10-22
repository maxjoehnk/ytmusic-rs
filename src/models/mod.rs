use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Thumbnail {
    pub height: usize,
    pub width: usize,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum AlbumType {
    Single,
    Album,
    EP,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ArtistReference {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AlbumReference {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PlaylistItem {
    pub video_id: String,
    pub title: String,
    pub artists: Vec<ArtistReference>,
    pub album: Option<AlbumReference>,
    pub is_available: bool,
    pub is_explicit: bool,
    pub duration: Option<String>,
    pub duration_seconds: Option<u64>,
    pub thumbnails: Vec<Thumbnail>,
}
