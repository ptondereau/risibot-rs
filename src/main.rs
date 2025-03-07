use std::env;
use std::time::Duration;

use bot::BotService;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use risibank::Risibank;
use teloxide::Bot;

mod bot;
mod handlers;
mod risibank;

#[tokio::main]
async fn main() {
    // Initialize logger
    pretty_env_logger::init();
    log::info!("Starting Risibot...");

    // Get token from environment variable
    let teloxide_key = env::var("TELOXIDE_TOKEN")
        .expect("You need a TELOXIDE_TOKEN env var set for this to work!");

    // Configure HTTP client with retry policy
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    let http_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(1))
        .pool_idle_timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(1)
        .build()
        .expect("failed to build http client");

    let retry_client = ClientBuilder::new(http_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    // Initialize RisiBank client
    let risibank_client = Risibank::new(retry_client);

    // Get webhook URL from environment variable
    let url =
        env::var("WEBHOOK_URL").expect("You need a WEBHOOK_URL env var set for this to work!");
    let webhook_url = reqwest::Url::parse(url.as_str()).expect("Invalid WEBHOOK_URL format");

    // Create bot service
    let bot_service = BotService {
        bot: Bot::new(teloxide_key),
        webhook_url,
        risibank: risibank_client,
    };

    // Get port from environment variable (provided by Upsun)
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    // Start the bot and listen for updates
    log::info!("Starting bot on port {}", port);

    // Here you should implement your bot's startup logic
    // This will depend on how your BotService is structured
    // For example:
    bot_service.start_webhook(port).await;

    // Keep the application running
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    log::info!("Shutting down gracefully");
}
