use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use zebedee_rust::ZebedeeClient;

use serde::{Deserialize, Serialize};

mod commands;
use commands::*;

mod battlemetrics;

#[derive(Serialize, Deserialize, Debug)]
struct CommandOptions {
    raw: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommandAttributes {
    command: String,
    options: CommandOptions,
}

#[derive(Serialize, Deserialize, Debug)]
struct RconCommand {
    #[serde(rename = "type")]
    type_field: String,
    attributes: CommandAttributes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RconData {
    data: RconCommand,
}

impl RconData {
    pub fn new(type_field: &str, command: &str, raw: &str) -> Self {
        RconData {
            data: RconCommand {
                type_field: type_field.to_string(),
                attributes: CommandAttributes {
                    command: command.to_string(),
                    options: CommandOptions {
                        raw: raw.to_string(),
                    },
                },
            },
        }
    }
}

pub struct Data {
    zbd: ZebedeeClient,
    api_client: reqwest::Client,
    bm_token: String,
    server_id: String,
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    let zbd_token = secret_store
        .get("ZBD_TOKEN")
        .context("'ZBD_TOKEN' was not found")?;

    let zebedee_client = ZebedeeClient::new().apikey(zbd_token).build();

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
            commands: vec![mint()],
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
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
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
