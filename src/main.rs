use anyhow::Context as _;
use battlemetrics::unmute_player;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use zebedee_rust::ZebedeeClient;

mod commands;
use commands::*;

mod battlemetrics;

pub struct Data {
    zbd: ZebedeeClient,
    api_client: reqwest::Client,
    bm_token: String,
    server_id: String,
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let zbd_token = secret_store
        .get("ZBD_TOKEN")
        .context("'ZBD_TOKEN' was not found")?;

    let zebedee_client = ZebedeeClient::new(zbd_token);

    let api_client = reqwest::Client::new();

    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let server_id = secret_store
        .get("SERVER_ID")
        .context("'SERVER_ID' was not found")?;
    let bm_token = secret_store
        .get("BM_TOKEN")
        .context("'BM_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![giveblood(), unmute()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    zbd: zebedee_client,
                    api_client,
                    bm_token,
                    server_id,
                })
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
