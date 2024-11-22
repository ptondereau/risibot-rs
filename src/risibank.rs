use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto};

const MAX_RESULTS: usize = 15;
const API_BASE_URL: &str = "https://risibank.fr/api/v0";

#[derive(Debug, Serialize, Deserialize)]
pub struct RisibankSearchResult {
    pub stickers: Vec<Sticker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sticker {
    pub risibank_link: Url,
    pub id: u64,
    pub ext: String,
}

#[derive(Debug, Clone)]
pub struct Risibank {
    client: Client,
}

impl Risibank {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn search(&self, query: &str) -> Result<RisibankSearchResult, reqwest::Error> {
        self.client
            .get(format!("{}/search", API_BASE_URL))
            .query(&[("search", query)])
            .send()
            .await?
            .json::<RisibankSearchResult>()
            .await
    }
}

/// Converts a `Sticker` to an `InlineQueryResult` enum
/// This adds a behavior when we found gifs, we notify Telegram that it's a gif
impl From<&Sticker> for InlineQueryResult {
    fn from(sticker: &Sticker) -> InlineQueryResult {
        let id = sticker.id.to_string();
        let url = sticker.risibank_link.clone();

        match sticker.ext.as_str() {
            "gif" => InlineQueryResult::Gif(InlineQueryResultGif::new(id, url.clone(), url)),
            _ => InlineQueryResult::Photo(InlineQueryResultPhoto::new(id, url.clone(), url)),
        }
    }
}

/// We take only the first 15th elements
impl From<RisibankSearchResult> for Vec<InlineQueryResult> {
    fn from(result: RisibankSearchResult) -> Vec<InlineQueryResult> {
        result
            .stickers
            .iter()
            .take(MAX_RESULTS)
            .map(Into::into)
            .collect()
    }
}
