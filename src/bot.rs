use std::sync::Arc;

use teloxide::{prelude::*, Bot};

use crate::{handlers::handle_inline, risibank::Risibank};

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

        let handler = Update::filter_inline_query().branch(dptree::endpoint(handle_inline));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .dependencies(dptree::deps![risibank])
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
