// use std::collections::HashSet;
// use std::env;
//
// use serenity::framework::standard::macros::command;
// use serenity::framework::standard::{Args, CommandResult};
// use serenity::framework::StandardFramework;
// use serenity::model::channel::Message;
// use serenity::model::gateway::Ready;
// use serenity::prelude::*;
// use serenity::{async_trait, http::Http};
// use tracing::{error, info};

use std::{collections::HashSet, env};

use serenity::{
    async_trait,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

use tracing::{error, info};

mod commands;

use commands::ping::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, _ctx: Context, _msg: Message) {}

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to find .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    // {
    //     let mut data = client.data.write().await;
    //     data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    // }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
