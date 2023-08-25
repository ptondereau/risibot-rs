use log::{debug, info};
use teloxide::{prelude::*, types::InlineQueryResult};

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

    let articles: Vec<InlineQueryResult> = result.into();

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
