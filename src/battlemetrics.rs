use serde::{Deserialize, Serialize};

use crate::commands::{Context, Error};

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

pub async fn mint_blood(
    name: Option<String>,
    amount: String,
    ctx: Context<'_>,
    api_client: &reqwest::Client,
) -> Result<(), Error> {
    if let Some(name) = name.clone() {
        let player_name = name;
        let short_name = "blood";

        let command_name = format!(
            r#"inventory.giveto "{}" "{}" {}"#,
            player_name, short_name, amount
        );
        println!("{:?}: Running Command: {}", player_name, command_name);

        let rcon_data = RconData::new("rconCommand", "raw", &command_name);

        let serialized_data = if let Ok(data) = serde_json::to_string(&rcon_data) {
            data
        } else {
            println!("error serializing data");
            return Ok(());
        };

        let server_id = ctx.data().server_id.clone();
        let url = format!(
            "https://api.battlemetrics.com/servers/{}/command",
            server_id
        );

        let bm_token = ctx.data().bm_token.clone();

        let res = api_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", bm_token))
            .header("Content-Type", "application/json")
            .body(serialized_data)
            .send()
            .await?;

        if res.status() == 200 {
            let reply = ctx
                .channel_id()
                .say(
                    ctx.http(),
                    format!("{} has been payed {} blood", player_name, amount),
                )
                .await;

            if let Err(e) = reply {
                println!("error: {}", e);
            }
            println!("{:?} blood minted.", player_name);
            Ok(())
        } else {
            println!("{:?} blood failed to mint.", player_name);
            let reply = ctx
                .channel_id()
                .say(
                    ctx.http(),
                    format!("Failed to pay {} blood to {}.", amount, player_name),
                )
                .await;

            if let Err(e) = reply {
                println!("errror: {}", e);
            }

            Ok(())
        }
    } else {
        println!("error minting blood");

        let reply = ctx
            .channel_id()
            .say(ctx.http(), "Failed to parse amount")
            .await;
        if let Err(e) = reply {
            println!("errror: {}", e);
        }
        Ok(())
    }
}

pub async fn unmute_player(
    name: Option<String>,
    ctx: Context<'_>,
    api_client: &reqwest::Client,
) -> Result<(), Error> {
    if let Some(name) = name.clone() {
        let player_name = name;
        let short_name = "unmute";

        let command_name = format!(r#"inventory.giveto "{}" "{}" "#, player_name, short_name);
        println!("{:?}: Running Command: {}", player_name, command_name);

        let rcon_data = RconData::new("rconCommand", "raw", &command_name);

        let serialized_data = if let Ok(data) = serde_json::to_string(&rcon_data) {
            data
        } else {
            println!("error serializing data");
            return Ok(());
        };

        let server_id = ctx.data().server_id.clone();
        let url = format!(
            "https://api.battlemetrics.com/servers/{}/command",
            server_id
        );

        let bm_token = ctx.data().bm_token.clone();

        let res = api_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", bm_token))
            .header("Content-Type", "application/json")
            .body(serialized_data)
            .send()
            .await?;

        if res.status() == 200 {
            let reply = ctx
                .channel_id()
                .say(ctx.http(), format!("{} has been unmuted", player_name))
                .await;

            if let Err(e) = reply {
                println!("error: {}", e);
            }
            println!("{:?} unmuted.", player_name);
            Ok(())
        } else {
            println!("Failed to unmute {:?}", player_name);
            let reply = ctx
                .channel_id()
                .say(
                    ctx.http(),
                    format!("Failed to unmute player {}.", player_name),
                )
                .await;

            if let Err(e) = reply {
                println!("errror: {}", e);
            }

            Ok(())
        }
    } else {
        println!("error unmuting player");

        let reply = ctx
            .channel_id()
            .say(ctx.http(), "Failed to unmute player")
            .await;
        if let Err(e) = reply {
            println!("errror: {}", e);
        }
        Ok(())
    }
}
