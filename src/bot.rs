use std::sync::Arc;

use log::{debug, info};
use teloxide::{prelude::*, update_listeners::webhooks, Bot};

use crate::{handlers::handle_inline, risibank::Risibank};

pub struct BotService {
    pub bot: Bot,
    pub risibank: Risibank,
    pub webhook_url: reqwest::Url,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let share_self = Arc::new(self);

        info!("Booting tokio tasks");

        tokio::spawn(async move {
            Arc::clone(&share_self)
                .start(&addr)
                .await
                .expect("An error ocurred while using the bot!");
        });

        Ok(())
    }
}

impl BotService {
    async fn start(
        &self,
        &addr: &std::net::SocketAddr,
    ) -> Result<(), shuttle_runtime::CustomError> {
        info!("Starting bot");
        let bot = self.bot.clone();
        let risibank = self.risibank.clone();

        let handler = Update::filter_inline_query().branch(dptree::endpoint(handle_inline));

        let listener = webhooks::axum(
            bot.clone(),
            webhooks::Options::new(addr, self.webhook_url.clone()),
        )
        .await
        .expect("failed to build listener");

        debug!(
            "Listener created with addr {:?} and webhook url {:?}",
            addr, self.webhook_url
        );

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .dependencies(dptree::deps![risibank])
            .build()
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;

        Ok(())
    }
}
