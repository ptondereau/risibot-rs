use log::{debug, info};
use teloxide::{
    prelude::*,
    types::{InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto},
};

use crate::risibank::Risibank;

pub async fn handle_inline(bot: Bot, q: InlineQuery, risibank: Risibank) -> ResponseResult<()> {
    if q.query.is_empty() {
        debug!("Empty query");
        return Ok(());
    }

    debug!("Query: {}", q.query);
    let result = risibank.search(q.query.as_str()).await;

    if let Err(err) = result {
        log::error!("Error in handler: {:?}", err);
        let response = bot.answer_inline_query(&q.id, []).send().await;
        if let Err(err) = response {
            log::error!("Error in handler: {:?}", err);
        }

        return Ok(());
    }

    let result = result.unwrap();

    if result.stickers.is_empty() {
        let response = bot.answer_inline_query(&q.id, []).send().await;
        if let Err(err) = response {
            log::error!("Error in handler: {:?}", err);
        }

        return Ok(());
    }

    let articles: Vec<InlineQueryResult> = result
        .stickers
        .iter()
        .take(15)
        .map(|sticker| match sticker.ext.as_str() {
            "gif" => {
                let article = InlineQueryResultGif::new(
                    sticker.id.to_string(),
                    sticker.risibank_link.clone(),
                    sticker.risibank_link.clone(),
                );
                InlineQueryResult::Gif(article)
            }
            _ => {
                let article = InlineQueryResultPhoto::new(
                    sticker.id.to_string(),
                    sticker.risibank_link.clone(),
                    sticker.risibank_link.clone(),
                );
                InlineQueryResult::Photo(article)
            }
        })
        .collect();

    let response = bot
        .answer_inline_query(&q.id, articles)
        .cache_time(0)
        .send()
        .await;
    if let Err(err) = response {
        log::error!("Error in handler: {:?}", err);
    }
    info!("Answered inline query");
    Ok(())
}
