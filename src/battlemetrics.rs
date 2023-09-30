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

        let serialized_data = serde_json::to_string(&rcon_data).unwrap();

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
            ctx.say(format!("{} has been payed {} blood", player_name, amount))
                .await?;
            println!("{:?} blood minted.", player_name);
            Ok(())
        } else {
            ctx.say(format!(
                "Failed to pay {} blood to {}. {}",
                amount,
                player_name,
                res.text().await?
            ))
            .await?;
            println!("{:?} blood failed to mint.", player_name);
            Ok(())
        }
    } else {
        println!("error minting blood");
        ctx.say("Failed to parse amount").await?;
        Ok(())
    }
}
