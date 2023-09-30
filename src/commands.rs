use qrcode_generator::QrCodeEcc;
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
                expires_in: 40,
                ..Default::default()
            };

            match zebedee_client.create_charge(&charge).await {
                Ok(invoice) => {
                    if let Some(data) = invoice.data {
                        let request_data = data.invoice.request;
                        match serde_json::to_string(&request_data) {
                            Ok(serialized_request_data) => {
                                let data = serialized_request_data.trim_matches('"').to_string();
                                let qr_invoice: Vec<u8> = qrcode_generator::to_png_to_vec(
                                    data.clone(),
                                    QrCodeEcc::Low,
                                    1024,
                                )
                                .unwrap();
                                let invoice_message = ctx.channel_id()
                                        .send_message(ctx.http(), |m| {
                                            m.embed(|e| {
                                                e.title("Blood Invoice");
                                                e.description(format!(
                                                    "{:?}, please pay {} sats to the following invoice to mint {} blood.",
                                                    name, amount, amount
                                                ));
                                                e.image("attachment://qr.png");
                                                e.field("Amount", &amount, false);
                                                e.field("Expires in", "40 seconds", false);
                                                e.field("Invoice: ", data.clone(), false);
                                                e
                                            });
                                            m.add_file((qr_invoice.as_slice(), "qr.png"));
                                            m
                                        })
                                        .await;

                                match invoice_message {
                                    Ok(_) => {
                                        println!("{:?}: invoice sent...", name);
                                    }
                                    Err(e) => {
                                        println!("{:?}: Failed to send invoice: {}", name, e);
                                    }
                                };

                                // for seconds_left in (1..=28).rev() {
                                //     sleep(Duration::from_millis(1000));

                                //     // Update the message
                                //     message
                                //         .edit(ctx.http(), |m| {
                                //             m.embed(|e| {
                                //                 e.title("Blood Invoice");
                                //                 e.description(format!(
                                //                     "Please pay {} sats to the following invoice.",
                                //                     amount
                                //                 ));
                                //                 e.image("attachment://qr.png");
                                //                 e.field("Amount", &amount, false);
                                //                 e.field(
                                //                     "Expires in",
                                //                     format!("{} seconds", seconds_left),
                                //                     false,
                                //                 );
                                //                 e
                                //             })
                                //         })
                                //         .await?;
                                // }
                            }
                            Err(e) => {
                                let reply = ctx
                                    .channel_id()
                                    .say(
                                        ctx.http(),
                                        format!("Failed to serialize request data: {}", e),
                                    )
                                    .await;

                                if let Err(e) = reply {
                                    println!("error: {}", e);
                                }
                            }
                        }

                        loop {
                            sleep(Duration::from_millis(1000));

                            match zebedee_client.get_charge(data.id.clone()).await {
                                Ok(charge) => {
                                    if let Some(data) = charge.data {
                                        match data.status.as_str() {
                                            "completed" => {
                                                println!(
                                                    "{:?}: payment completed...minting blood...",
                                                    name
                                                );
                                                let mint = mint_blood(
                                                    name.clone(),
                                                    amount.clone(),
                                                    ctx,
                                                    api_client,
                                                )
                                                .await;
                                                if let Err(e) = mint {
                                                    println!("mint error: {}", e);
                                                }
                                                break;
                                            }
                                            "expired" => {
                                                let reply = ctx
                                                    .channel_id()
                                                    .say(ctx.http(), "payment expired")
                                                    .await;

                                                if let Err(e) = reply {
                                                    println!("error: {}", e);
                                                }
                                                println!("{:?}: payment expired.", name);
                                                break;
                                            }
                                            "error" => {
                                                let reply = ctx
                                                    .channel_id()
                                                    .say(ctx.http(), "payment error")
                                                    .await;

                                                if let Err(e) = reply {
                                                    println!("error: {}", e);
                                                }
                                                break;
                                            }
                                            _ => {
                                                println!("{:?}: Waiting for payment...", name);
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
                        println!("invoice data error...");
                    }
                }
                Err(e) => {
                    let reply = ctx
                        .channel_id()
                        .say(ctx.http(), format!("Failed to create charge: {}", e))
                        .await;
                    if let Err(e) = reply {
                        println!("error: {}", e);
                    }
                }
            }
        } else {
            let reply = ctx
                .channel_id()
                .say(ctx.http(), "Please enter a valid number for the amount.")
                .await;
            if let Err(e) = reply {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
