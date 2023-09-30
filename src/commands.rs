use std::{thread::sleep, time::Duration};

use zebedee_rust::charges::*;

use crate::{battlemetrics::mint_blood, Data};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with ln invoice
#[poise::command(prefix_command, track_edits, aliases("amount, name"), slash_command)]
pub async fn mint(
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

                        loop {
                            sleep(Duration::from_millis(5000));

                            match zebedee_client.get_charge(data.id.clone()).await {
                                Ok(charge) => {
                                    if let Some(data) = charge.data {
                                        match data.status.as_str() {
                                            "completed" => {
                                                mint_blood(
                                                    name.clone(),
                                                    amount.clone(),
                                                    ctx,
                                                    api_client,
                                                )
                                                .await?;
                                                println!("payment completed.");
                                                break;
                                            }
                                            "expired" => {
                                                ctx.say("payment expired".to_string()).await?;
                                                println!("payment expired.");
                                                break;
                                            }
                                            "error" => {
                                                ctx.say("payment error".to_string()).await?;
                                                break;
                                            }
                                            _ => {
                                                println!("Waiting for payment...")
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("error...");
                                    break;
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
