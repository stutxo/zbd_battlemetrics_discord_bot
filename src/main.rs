use std::{thread::sleep, time::Duration};

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_runtime::tracing_subscriber::fmt::format;
use shuttle_secrets::SecretStore;
use zebedee_rust::{charges::*, ZebedeeClient};

use serde::{Deserialize, Serialize};
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
struct RconData {
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

struct Data {
    zbd: ZebedeeClient,
    api_client: reqwest::Client,
    bm_token: String,
    server_id: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with ln invoice
#[poise::command(prefix_command, track_edits, aliases("amount, name"), slash_command)]
async fn mint(
    ctx: Context<'_>,
    #[description = "blood amount to buy"] amount: Option<String>,
    #[description = "In game name"] name: Option<String>,
) -> Result<(), Error> {
    let zebedee_client = &ctx.data().zbd;
    let api_client = &ctx.data().api_client;

    if let Some(amount) = amount {
        if let Ok(mut num) = amount.parse::<i32>() {
            num *= 1000;
            let new_amount = num.to_string();

            let charge = Charge {
                amount: new_amount,
                description: "Buy Blood".to_string(),
                ..Default::default()
            };

            match zebedee_client.create_charge(&charge).await {
                Ok(invoice) => {
                    if let Some(data) = invoice.data {
                        let request_data = data.invoice.request;
                        match serde_json::to_string(&request_data) {
                            Ok(serialized_request_data) => {
                                ctx.say(serialized_request_data).await?;
                            }
                            Err(e) => {
                                ctx.say(format!("Failed to serialize request data: {}", e))
                                    .await?;
                            }
                        }

                        let mut payed = false;

                        while !payed {
                            sleep(Duration::from_millis(100));

                            if let Ok(charge) = zebedee_client.get_charge(data.id.clone()).await {
                                if let Some(data) = charge.data {
                                    if data.status == "completed" {
                                        payed = true;

                                        if let Some(name) = name.clone() {
                                            let player_name = name;
                                            let short_name = "blood";

                                            let command_name = format!(
                                                r#"inventory.giveto "{}" "{}" {}"#,
                                                player_name, short_name, amount
                                            );
                                            println!("Command: {}", command_name);

                                            let rcon_data =
                                                RconData::new("rconCommand", "raw", &command_name);

                                            let serialized_data =
                                                serde_json::to_string(&rcon_data).unwrap();

                                            let server_id = ctx.data().server_id.clone();
                                            let url = format!(
                                                "https://api.battlemetrics.com/servers/{}/command",
                                                server_id
                                            );

                                            let bm_token = ctx.data().bm_token.clone();

                                            let res = api_client
                                                .post(&url)
                                                .header(
                                                    "Authorization",
                                                    format!("Bearer {}", bm_token),
                                                )
                                                .header("Content-Type", "application/json")
                                                .body(serialized_data)
                                                .send()
                                                .await?;

                                            if res.status() == 200 {
                                                ctx.say(format!(
                                                    "{} has been payed {} blood",
                                                    player_name, amount
                                                ))
                                                .await?;
                                            } else {
                                                ctx.say(format!(
                                                    "Failed to pay {} blood to {}. {}",
                                                    amount,
                                                    player_name,
                                                    res.text().await?
                                                ))
                                                .await?;
                                            }
                                        }
                                        if data.status == "expired" {
                                            payed = true;
                                            ctx.say("payment expired".to_string()).await?;
                                        }
                                        if data.status == "error" {
                                            payed = true;
                                            ctx.say("payment error".to_string()).await?;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        ctx.say("Invoice data is empty.").await?;
                    }
                }
                Err(e) => {
                    ctx.say(format!("Failed to create charge: {}", e)).await?;
                }
            }
        } else {
            ctx.say("Please enter a valid number for the amount.")
                .await?;
        }
    }

    Ok(())
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
