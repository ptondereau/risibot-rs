use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto};
use thiserror::Error;

const MAX_RESULTS: usize = 15;
const API_BASE_URL: &str = "https://risibank.fr/api/v0";

#[derive(Error, Debug)]
pub enum RisibankError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

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

    pub async fn search(&self, query: &str) -> Result<RisibankSearchResult, RisibankError> {
        let response = self
            .client
            .get(format!("{}/search", API_BASE_URL))
            .query(&[("search", query)])
            .send()
            .await?;

        match response.status() {
            status if status.is_success() => {
                let result = response.json::<RisibankSearchResult>().await?;
                Ok(result)
            }
            status if status == StatusCode::TOO_MANY_REQUESTS => Err(RisibankError::RateLimit),
            status => Err(RisibankError::InvalidResponse(format!(
                "Unexpected status code: {}",
                status
            ))),
        }
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
