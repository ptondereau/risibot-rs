use std::time::Duration;

use bot::BotService;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use shuttle_runtime::SecretStore;
use teloxide::Bot;

mod bot;
mod handlers;
mod risibank;

#[shuttle_runtime::main]
async fn init(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    let teloxide_key = secret_store
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

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

    let risibank_client = risibank::Risibank::new(retry_client);
    let url = secret_store
        .get("WEBHOOK_URL")
        .expect("You need a WEBHOOK_URL key set for this to work!");
    let url = reqwest::Url::parse(url.as_str()).unwrap();

    Ok(BotService {
        bot: Bot::new(teloxide_key),
        webhook_url: url,
        risibank: risibank_client,
    })
}
