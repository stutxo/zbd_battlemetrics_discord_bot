use poise::serenity_prelude::{CreateAttachment, CreateEmbed, CreateMessage};
use qrcode_generator::QrCodeEcc;
use std::{thread::sleep, time::Duration};
use zebedee_rust::charges::*;

use crate::{
    battlemetrics::{mint_blood, unmute_player},
    Data,
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with ln invoice
#[poise::command(prefix_command, track_edits, aliases("amount, name"), slash_command)]
pub async fn giveblood(
    ctx: Context<'_>,
    #[description = "blood amount to buy"] amount: Option<String>,
    #[description = "In game name"] name: Option<String>,
) -> Result<(), Error> {
    let zebedee_client = &ctx.data().zbd;
    let api_client = &ctx.data().api_client;

    let name_str = name.as_deref().unwrap_or("User");

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
                    if let Some(request_data) = invoice.data {
                        let requested_invoice =
                            if let Some(requested_invoice) = request_data.invoice {
                                requested_invoice
                            } else {
                                let reply = ctx
                                    .channel_id()
                                    .say(ctx.http(), "Failed to get invoice data.")
                                    .await;
                                if let Err(e) = reply {
                                    println!("error: {}", e);
                                }
                                return Ok(());
                            };
                        match serde_json::to_string(&requested_invoice.request) {
                            Ok(serialized_request_data) => {
                                let data = serialized_request_data.trim_matches('"').to_string();
                                let qr_invoice: Vec<u8> = qrcode_generator::to_png_to_vec(
                                    data.clone(),
                                    QrCodeEcc::Low,
                                    1024,
                                )
                                .unwrap();

                                let description = format!("{:?}, please pay {} sats to the following invoice to give {} blood.", name_str, amount, amount);

                                let embed = CreateEmbed::new()
                                    .title("Blood Invoice")
                                    .description(description)
                                    .fields(vec![
                                        ("Amount", amount.clone(), false),
                                        ("Expires in", "40 seconds".to_string(), false),
                                        ("Invoice: ", data.clone(), false),
                                    ]);

                                let attachment =
                                    CreateAttachment::bytes(qr_invoice.as_slice(), "qr.png");

                                let builder =
                                    CreateMessage::new().embed(embed).add_file(attachment);

                                let invoice_message =
                                    ctx.channel_id().send_message(&ctx.http(), builder).await;

                                match invoice_message {
                                    Ok(_) => {
                                        println!("{:?}: invoice sent...", name_str);
                                    }
                                    Err(e) => {
                                        println!("{:?}: Failed to send invoice: {}", name_str, e);
                                    }
                                };
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

                            match zebedee_client.get_charge(request_data.id.clone()).await {
                                Ok(charge) => {
                                    if let Some(data) = charge.data {
                                        match data.status.as_str() {
                                            "completed" => {
                                                println!(
                                                    "{:?}: payment completed...sending blood...",
                                                    name_str
                                                );
                                                let give_blood = mint_blood(
                                                    name.clone(),
                                                    amount.clone(),
                                                    ctx,
                                                    api_client,
                                                )
                                                .await;
                                                if let Err(e) = give_blood {
                                                    println!("sending blood error: {}", e);
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
                                                println!("{:?}: payment expired.", name_str);
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
                                                println!("{:?}: Waiting for payment...", name_str);
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

#[poise::command(prefix_command, track_edits, aliases("name"), slash_command)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "In game name"] name: Option<String>,
) -> Result<(), Error> {
    let zebedee_client = &ctx.data().zbd;
    let api_client = &ctx.data().api_client;

    let name_str = name.as_deref().unwrap_or("User");

    //how much should this be?
    let new_amount = 1000;

    let charge = Charge {
        amount: new_amount.to_string(),
        description: "unmute player".to_string(),
        expires_in: 40,
        ..Default::default()
    };

    match zebedee_client.create_charge(&charge).await {
        Ok(invoice) => {
            if let Some(request_data) = invoice.data {
                let requested_invoice = if let Some(requested_invoice) = request_data.invoice {
                    requested_invoice
                } else {
                    let reply = ctx
                        .channel_id()
                        .say(ctx.http(), "Failed to get invoice data.")
                        .await;
                    if let Err(e) = reply {
                        println!("error: {}", e);
                    }
                    return Ok(());
                };
                match serde_json::to_string(&requested_invoice.request) {
                    Ok(serialized_request_data) => {
                        let data = serialized_request_data.trim_matches('"').to_string();
                        let qr_invoice: Vec<u8> =
                            qrcode_generator::to_png_to_vec(data.clone(), QrCodeEcc::Low, 1024)
                                .unwrap();

                        let description = format!(
                            "{:?}, please pay {} sats to the following invoice to unmute {}.",
                            name_str, new_amount, name_str
                        );

                        let embed = CreateEmbed::new()
                            .title("Blood Invoice")
                            .description(description)
                            .fields(vec![
                                ("Amount", new_amount.to_string().clone(), false),
                                ("Expires in", "40 seconds".to_string(), false),
                                ("Invoice: ", data.clone(), false),
                            ]);

                        let attachment = CreateAttachment::bytes(qr_invoice.as_slice(), "qr.png");

                        let builder = CreateMessage::new().embed(embed).add_file(attachment);

                        let invoice_message =
                            ctx.channel_id().send_message(&ctx.http(), builder).await;

                        match invoice_message {
                            Ok(_) => {
                                println!("{:?}: invoice sent...", name_str);
                            }
                            Err(e) => {
                                println!("{:?}: Failed to send invoice: {}", name_str, e);
                            }
                        };
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

                    match zebedee_client.get_charge(request_data.id.clone()).await {
                        Ok(charge) => {
                            if let Some(data) = charge.data {
                                match data.status.as_str() {
                                    "completed" => {
                                        println!(
                                            "{:?}: payment completed...unmuting player...",
                                            name_str
                                        );
                                        let unmute_player =
                                            unmute_player(name.clone(), ctx, api_client).await;
                                        if let Err(e) = unmute_player {
                                            println!("unmuting player error: {}", e);
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
                                        println!("{:?}: payment expired.", name_str);
                                        break;
                                    }
                                    "error" => {
                                        let reply =
                                            ctx.channel_id().say(ctx.http(), "payment error").await;

                                        if let Err(e) = reply {
                                            println!("error: {}", e);
                                        }
                                        break;
                                    }
                                    _ => {
                                        println!("{:?}: Waiting for payment...", name_str);
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

    Ok(())
}
