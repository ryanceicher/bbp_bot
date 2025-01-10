mod commands;
mod dataaccess;

use poise::serenity_prelude as serenity;
use std::env;
use std::sync::Arc;
use poise::futures_util::lock::Mutex;
use serenity::prelude::TypeMapKey;
use crate::dataaccess::postgres_service::PostgresService;

struct PostgresServiceContainer;

impl TypeMapKey for PostgresServiceContainer{
    type Value = Arc<Mutex<PostgresService>>;
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
pub struct Data {
    db: Arc<Mutex<PostgresService>>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let constr = env::var("PG_CONNECTION_STRING").expect("Expected a connection string for postgres.");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::bbp_commands::bbp_add_command(),
                commands::bbp_commands::gbp_add_command(),
                commands::bbp_commands::add_user_command(),
                commands::bbp_commands::bbp_forgive_command(),
                commands::bbp_commands::leaderboard_command(),
                commands::bbp_commands::history_command(),
            ],
            initialize_owners: true,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let db = PostgresService::new(&constr).await
                    .expect("Couldn't build database connection");
                let data = Data {
                    db: Arc::new(Mutex::new(db)),
                };
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();
    
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    
    client.unwrap().start().await.unwrap();
}