use std::fmt;

use reqwest::{Client, Url};
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RisibankSearchResult {
    pub stickers: Vec<Sticker>,
}

#[derive(Debug, Serialize, Deserialize)]
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

fn deserialize_stickers<'de, D>(deserializer: D) -> Result<Vec<Sticker>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StickersVisitor;

    impl<'de> Visitor<'de> for StickersVisitor {
        type Value = Vec<Sticker>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut vec: Vec<Sticker> = Vec::with_capacity(15);

            while let Some(value) = seq.next_element()? {
                vec.push(value);
                if vec.len() == 15 {
                    break;
                }
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(StickersVisitor)
}
