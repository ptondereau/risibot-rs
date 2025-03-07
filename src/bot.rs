use teloxide::{
    prelude::*,
    update_listeners::webhooks::{self, axum},
    Bot,
};

use crate::{handlers::handle_inline, risibank::Risibank};

pub struct BotService {
    pub bot: Bot,
    pub risibank: Risibank,
    pub webhook_url: reqwest::Url,
}

impl BotService {
    pub async fn start_webhook(&self, port: u16) {
        log::info!("Setting webhook to {}", self.webhook_url);

        let bot: Bot = self.bot.clone();
        let risibank = self.risibank.clone();

        // Set up the webhook
        bot.set_webhook(self.webhook_url.clone())
            .await
            .expect("Failed to set webhook");

        let addr = ([127, 0, 0, 1], port).into();

        // Create webhook server
        let handler = Update::filter_inline_query().branch(dptree::endpoint(handle_inline));

        let listener = axum(
            bot.clone(),
            webhooks::Options::new(addr, self.webhook_url.clone()),
        )
        .await
        .expect("Failed to create webhook listener");

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .dependencies(dptree::deps![risibank])
            .build()
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await
    }
}
