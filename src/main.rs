use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use zebedee_rust::{charges::*, ZebedeeClient};

struct Data {
    zbd: ZebedeeClient,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with ln invoice
#[poise::command(prefix_command, track_edits, aliases("amount"), slash_command)]
async fn mint(
    ctx: Context<'_>,
    #[description = "blood amount to buy"] amount: Option<String>,
) -> Result<(), Error> {
    let zebedee_client = &ctx.data().zbd;

    if let Some(amount) = amount {
        if let Ok(mut num) = amount.parse::<i32>() {
            num *= 1000;
            let new_amount = num.to_string();

            let charge = Charge {
                amount: new_amount,
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
                            match zebedee_client.get_charge(data.id.clone()).await {
                                Ok(charge) => {
                                    if let Some(data) = charge.data {
                                        if data.status == "paid" {
                                            payed = true;
                                            ctx.say("payment accepted".to_string()).await?;
                                        }
                                    }
                                }
                                Err(e) => {
                                    ctx.say(format!("Failed to get charge: {}", e)).await?;
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

    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

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
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
