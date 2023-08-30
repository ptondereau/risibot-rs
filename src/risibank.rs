use std::borrow::Cow;

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto};

#[derive(Debug, Serialize, Deserialize)]
pub struct RisibankSearchResult {
    pub stickers: Vec<Sticker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sticker {
    pub risibank_link: Url,
    pub id: u64,
    pub ext: Cow<'static, str>,
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
            .get("https://risibank.fr/api/v0/search")
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
        match sticker.ext.as_ref() {
            "gif" => InlineQueryResult::Gif(InlineQueryResultGif::new(
                sticker.id.to_string(),
                sticker.risibank_link.clone(),
                sticker.risibank_link.clone(),
            )),
            _ => InlineQueryResult::Photo(InlineQueryResultPhoto::new(
                sticker.id.to_string(),
                sticker.risibank_link.clone(),
                sticker.risibank_link.clone(),
            )),
        }
    }
}

/// We take only the first 15th elements
impl From<RisibankSearchResult> for Vec<InlineQueryResult> {
    fn from(result: RisibankSearchResult) -> Vec<InlineQueryResult> {
        result.stickers.iter().take(15).map(|s| s.into()).collect()
    }
}
