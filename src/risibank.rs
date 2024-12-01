use reqwest::{StatusCode, Url};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto};
use thiserror::Error;

const MAX_RESULTS: usize = 15;
const API_BASE_URL: &str = "https://risibank.fr/api/v0";

#[derive(Error, Debug)]
pub enum RisibankError {
    #[error("HTTP request failed: {0}")]
    RequestMiddlewareError(#[from] reqwest_middleware::Error),

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
    client: ClientWithMiddleware,
    base_url: String,
}

impl Risibank {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self {
            client,
            base_url: API_BASE_URL.to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<RisibankSearchResult, RisibankError> {
        let response = self
            .client
            .get(format!("{}/search", self.base_url))
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use reqwest_middleware::ClientBuilder;
    use std::str::FromStr;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Helper function to create a test sticker
    fn create_test_sticker(id: u64, ext: &str) -> Sticker {
        Sticker {
            risibank_link: Url::from_str(&format!("https://risibank.fr/media/{}.{}", id, ext))
                .unwrap(),
            id,
            ext: ext.to_string(),
        }
    }

    #[tokio::test]
    async fn test_search_success() {
        let mock_server = MockServer::start().await;

        // Prepare mock response
        let response = RisibankSearchResult {
            stickers: vec![create_test_sticker(1, "jpg"), create_test_sticker(2, "gif")],
        };

        Mock::given(method("GET"))
            .and(path("/search")) // Updated path
            .and(query_param("search", "test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let client = ClientBuilder::new(client).build();
        let mut risibank = Risibank::new(client);
        // Use a test-specific URI instead of the constant
        risibank.base_url = mock_server.uri();

        let result = risibank.search("test").await.unwrap();

        assert_eq!(result.stickers.len(), 2);
        assert_eq!(result.stickers[0].id, 1);
        assert_eq!(result.stickers[0].ext, "jpg");
    }

    #[tokio::test]
    async fn test_rate_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search")) // Updated path
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let client = ClientBuilder::new(client).build();
        let mut risibank = Risibank::new(client);
        // Use a test-specific URI instead of the constant
        risibank.base_url = mock_server.uri();

        let result = risibank.search("test").await;

        assert!(matches!(result.unwrap_err(), RisibankError::RateLimit));
    }

    #[tokio::test]
    async fn test_sticker_to_inline_query_result() {
        let photo_sticker = create_test_sticker(1, "jpg");
        let gif_sticker = create_test_sticker(2, "gif");

        let photo_result = InlineQueryResult::from(&photo_sticker);
        let gif_result = InlineQueryResult::from(&gif_sticker);

        assert!(matches!(photo_result, InlineQueryResult::Photo(_)));
        assert!(matches!(gif_result, InlineQueryResult::Gif(_)));
    }

    #[tokio::test]
    async fn test_search_result_conversion() {
        let stickers = vec![
            create_test_sticker(1, "jpg"),
            create_test_sticker(2, "gif"),
            create_test_sticker(3, "png"),
        ];

        let search_result = RisibankSearchResult { stickers };
        let inline_results: Vec<InlineQueryResult> = search_result.into();

        assert_eq!(inline_results.len(), 3);
    }

    #[tokio::test]
    async fn test_max_results_limit() {
        let mut stickers = Vec::new();
        for i in 0..20 {
            stickers.push(create_test_sticker(i, "jpg"));
        }

        let search_result = RisibankSearchResult { stickers };
        let inline_results: Vec<InlineQueryResult> = search_result.into();

        assert_eq!(inline_results.len(), MAX_RESULTS);
    }
}
