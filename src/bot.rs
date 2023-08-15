use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{InlineQuery, InlineQueryResult, InlineQueryResultGif, InlineQueryResultPhoto},
    Bot,
};

use crate::risibank::Risibank;

pub struct BotService {
    pub bot: Bot,
    pub risibank: Risibank,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let share_self = Arc::new(self);

        tokio::spawn(async move {
            Arc::clone(&share_self)
                .start()
                .await
                .expect("An error ocurred while using the bot!");
        });

        Ok(())
    }
}

impl BotService {
    async fn start(&self) -> Result<(), shuttle_runtime::CustomError> {
        let bot = self.bot.clone();
        let risibank = self.risibank.clone();

        let handler = Update::filter_inline_query().branch(dptree::endpoint(
            |bot: Bot, q: InlineQuery, risibank: Risibank| async move {
                if q.query.is_empty() {
                    return respond(());
                }

                let result = risibank.search(q.query.as_str()).await;

                if let Err(err) = result {
                    log::error!("Error in handler: {:?}", err);
                    let response = bot.answer_inline_query(&q.id, []).send().await;
                    if let Err(err) = response {
                        log::error!("Error in handler: {:?}", err);
                    }

                    return respond(());
                }

                let result = result.unwrap();

                if result.stickers.is_empty() {
                    let response = bot.answer_inline_query(&q.id, []).send().await;
                    if let Err(err) = response {
                        log::error!("Error in handler: {:?}", err);
                    }

                    return respond(());
                }

                let articles: Vec<InlineQueryResult> = result
                    .stickers
                    .iter()
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

                let response = bot.answer_inline_query(&q.id, articles).send().await;
                if let Err(err) = response {
                    log::error!("Error in handler: {:?}", err);
                }
                respond(())
            },
        ));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .dependencies(dptree::deps![risibank])
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
