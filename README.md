# Discord Bot For Game Servers To Accept Bitcoin. Uses ZBD & Battlemetrics APIs, Written In Rust And Deployed With Shuttle.

This discord bot connects your game server to your Bitcoin lightning wallet via the Battlemetrics and ZBD APIs. Any server command can be put behind a lightning bitcoin paywall, enabling game server owners to monetize their server through the sale of server VIP perks, ingame items & more.

To get started:

1. Clone this github repository to your local machine.

`git clone https://github.com/stum0/zbd_discord_bot.git`

2. Create a `Secrets.toml` file and copy your [ZBD](https://zbd.dev), [Battlemetrics](https://www.battlemetrics.com/developers/documentation) & [Discord](https://discord.com/developers/docs/getting-started) bot secrets and your Battlemetrics server id:
```
DISCORD_TOKEN = 'discord'

ZBD_TOKEN = 'zbd'

SERVER_ID = 'id'

BM_TOKEN = 'bm token'
```
3. Create a `.gitignore` file to omit your `Secrets.toml` from version control.

4. [Install](https://docs.shuttle.rs/getting-started/installation) [Shuttle](https://docs.shuttle.rs/introduction/welcome).

`cargo install cargo-shuttle`

5. Login to shuttle.

`cargo shuttle login`

6. Start your project with [idle minutes](https://docs.shuttle.rs/getting-started/idle-projects) set to 0 to keep it alive.

`cargo shuttle project start --idle-minutes 0`

7. Deploy your discord bot.

` cargo shuttle deploy --allow-dirty`

In this example, at the time of writing, the only command provided is `/mint`. It takes `amount` and `player` as inputs and responds with a `lightning invoice` for the amount in satoshis (sats). The player name must match the ingame name of an online player. Once the invoice has been paid it will execute the ingame command `giveitem blood amount player` for the game Rust; this gives the player the amount of blood corresponding to the amount of sats they have paid. This is a specific use case for the [Orange](https://orangem.art) Rust server.

Another useful server command that could be put behind a lightning bitcoin paywall is `oxide.usergroup add user group` to sell access for players to VIP groups with ingame perks (e.g. permission to use plugins, queue skip etc). This would work for games that use the [uMod](https://umod.org/documentation/plugins/permissions) plugin platform.

The Battlemetrics RCON supports 30+ games, including Battlebit, CS:GO, DayZ, Minecraft, Rust, Team Fortress 2 & Valheim. [How to execute RCON commands](https://github.com/BloodfallenTear/BMSharp/blob/master/docs/RCON.md) via the Battlemetrics API.

`https://api.battlemetrics.com/servers/$SERVER_ID/command`

```
{
   "data":{
      "type":"rconCommand",
      "attributes":{
         "command":"raw",
         "options":{
            "raw":"$COMMAND_NAME"
         }
      }
   }
}
```
