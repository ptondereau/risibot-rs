use std::{convert::Infallible, sync::Arc};

use log::info;
use teloxide::{
    prelude::*,
    update_listeners::{
        webhooks::{axum, Options},
        UpdateListener,
    },
    Bot,
};

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

        let options = Options::new(addr, share_self.webhook_url.clone());
        let update_listeners = axum(share_self.bot.clone(), options)
            .await
            .expect("failed to bind");

        share_self.start(update_listeners).await?;

        Ok(())
    }
}

impl BotService {
    async fn start(
        &self,
        listener: impl UpdateListener<Err = Infallible>,
    ) -> Result<(), shuttle_runtime::CustomError> {
        info!("Starting bot");
        let bot = self.bot.clone();
        let risibank = self.risibank.clone();

        let handler = Update::filter_inline_query().branch(dptree::endpoint(handle_inline));

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
