use std::time::Duration;

use bot::BotService;
use shuttle_secrets::SecretStore;
use teloxide::Bot;

mod bot;
mod handlers;
mod risibank;

#[shuttle_runtime::main]
async fn init(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    let teloxide_key = secret_store
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    let http_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(1))
        .build()
        .expect("failed to build http client");
    let risibank_client = risibank::Risibank::new(http_client);
    let url = secret_store
        .get("WEBHOOK_URL")
        .expect("You need a WEBHOOK_URL key set for this to work!");
    let url = reqwest::Url::parse(&url).unwrap();

    Ok(BotService {
        bot: Bot::new(teloxide_key),
        webhook_url: url,
        risibank: risibank_client,
    })
}
