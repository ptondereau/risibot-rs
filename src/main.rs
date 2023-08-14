use bot::BotService;
use shuttle_secrets::SecretStore;
use teloxide::Bot;

mod bot;

#[shuttle_runtime::main]
async fn init(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    let teloxide_key = secret_store
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    Ok(BotService {
        bot: Bot::new(teloxide_key),
    })
}
