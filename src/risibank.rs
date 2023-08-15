use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

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
            .get("https://risibank.fr/api/v0/search")
            .query(&[("search", query)])
            .send()
            .await?
            .json::<RisibankSearchResult>()
            .await
    }
}
