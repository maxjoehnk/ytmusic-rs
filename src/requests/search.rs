use serde::{Deserialize, Serialize};

use crate::{YoutubeMusicClient, YoutubeMusicError};
use crate::json_ext::JsonExt;

impl YoutubeMusicClient {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, YoutubeMusicError> {
        let body = SearchRequest {
            query
        };

        let response: serde_json::Value = self.get("search", &body).await?;
        let content = response.nav(&[
            &"contents",
            &"tabbedSearchResultsRenderer",
            &"tabs",
            &0,
            &"tabRenderer",
            &"content",
            &"sectionListRenderer",
            &"contents"
        ]).ok_or(YoutubeMusicError::ApiError)?;
        let contents: Vec<SectionListItem> = serde_json::from_value(content)?;

        println!("{:?}", contents);

        todo!()
    }
}

fn get_search_result_content(value: serde_json::Value) -> Option<serde_json::Value> {
    value
        .get("contents")?
        .get("tabbedSearchResultsRenderer")?
        .get("tabs")?
        .get(0)?
        .get("tabRenderer")?
        .get("content")?
        .get("sectionListRenderer")?
        .get("contents")
        .cloned()
}


#[derive(Deserialize)]
pub enum SearchResult {}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SectionListItem {
    ItemSectionRenderer {},
    MusicShelfRenderer {
        contents: Vec<MusicResponsiveListItemRenderer>
    },
}

#[derive(Debug, Clone, Deserialize)]
struct MusicResponsiveListItemRenderer {}

#[derive(Debug, Clone, Serialize)]
struct SearchRequest<'a> {
    query: &'a str,
}
