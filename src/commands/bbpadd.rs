use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

use crate::PostgresServiceContainer;
use crate::dataaccess::postgres_service;
use crate::commands::commands::COMMANDS_COMMAND_COMMAND;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^<@\d+>$").unwrap();
}

#[command]
#[bucket = "complicated"]
#[sub_commands(bbpadd_command, forgive, commands_command)]
async fn bbp(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[bucket = "complicated"]
#[sub_commands(gbpadd_command)]
async fn gbp(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add-user")]
async fn bbpadduser_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (issuer, target, description) = parse_command_mention_and_description(msg, args)?;

    let ranked_user = add_bbp(ctx, issuer, target, description).await?
        .ok_or_else(|| CommandError::from("Ranked user not found"))?;

    msg.channel_id.say(&ctx.http,
        format!("{}(#{}) now has {} bbps.",
            &ranked_user.friendly_name.unwrap(),
            &ranked_user.rank.unwrap(),
            &ranked_user.points)).await?;

    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn bbpadd_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (issuer, target, description) = parse_command_mention_and_description(msg, args)?;

    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    let issuing_user = db.get_user_by_discord_mention(&issuer).await?
        .ok_or_else(CommandError::from("Issuing user not found"))?;

    let target_user = db.get_user_by_discord_mention(&target).await?
        .ok_or_else(CommandError::from("Target user not found"))?;

    db.add_bbp_to_user(&target_user, &issuing_user, &description).await?;

    let ranked_user = db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await?
        .ok_or_else(CommandError::from("Ranked user not found"))?;

    msg.channel_id.say(&ctx.http, 
        format!("{}(#{}) now has {} bbps.",
            &ranked_user.friendly_name.unwrap(),
            &ranked_user.rank.unwrap(),
            &ranked_user.points)).await?;

    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn gbpadd_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (issuer, target, description) = parse_command_mention_and_description(msg, args)?;

    if issuer == target {
        let ranked_user = add_bbp(ctx, target.clone(), target, "Attempting to give themselves a GBP 😡".to_string()).await?
            .ok_or_else(CommandError::from("Ranked user not found"))?;

        msg.channel_id.say(&ctx.http,
            format!("😡 trying to give yourself a gbp? Thats a bbp for you. {}(#{}) now has {} bbps.",
                &ranked_user.friendly_name.unwrap(),
                &ranked_user.rank.unwrap(),
                &ranked_user.points)).await?;
    } else {
        let ranked_user = add_gbp(ctx, issuer, target, description).await?
            .ok_or_else(CommandError::from("Ranked user not found"))?;

        msg.channel_id.say(&ctx.http,
            format!("{}(#{}) now has {} bbps.",
                &ranked_user.friendly_name.unwrap(),
                &ranked_user.rank.unwrap(),
                &ranked_user.points)).await?;
    }

    Ok(())
}

async fn add_bbp(ctx: &Context, issuer: String, target: String, description: String) -> Result<Option<postgres_service::User>, Box<dyn std::error::Error + Send + Sync>> {
    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    let issuing_user = db.get_user_by_discord_mention(&issuer).await?
        .ok_or_else(|| "Issuing user not found")?;

    let target_user = db.get_user_by_discord_mention(&target).await?
        .ok_or_else(|| "Target user not found")?;

    db.add_bbp_to_user(&target_user, &issuing_user, &description).await?;

    let ranked_user = db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await?
        .ok_or_else(|| "Ranked user not found")?;

    Ok(Some(ranked_user))
}

async fn add_gbp(ctx: &Context, issuer: String, target: String, description: String) -> Result<Option<postgres_service::User>, Box<dyn std::error::Error + Send + Sync>> {
    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    let issuing_user = db.get_user_by_discord_mention(&issuer).await?
        .ok_or_else(|| "Issuing user not found")?;

    let target_user = db.get_user_by_discord_mention(&target).await?
        .ok_or_else(|| "Target user not found")?;

    db.add_gbp_to_user(&target_user, &issuing_user, &description).await?;

    let ranked_user = db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await?
        .ok_or_else(|| "Ranked user not found")?;

    Ok(Some(ranked_user))
}

#[command]
#[bucket = "complicated"]
#[aliases("forgive")]
async fn forgive(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (issuer, target, _) = parse_command_mention_only(msg, args)?;

    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    let issuing_user = db.get_user_by_discord_mention(&issuer).await?
        .ok_or_else(|| CommandError::from("Issuing user not found"))?;

    let target_user = db.get_user_by_discord_mention(&target).await?
        .ok_or_else(|| CommandError::from("Target user not found"))?;

    let description = db.forgive_user(&target_user, &issuing_user).await?
        .ok_or_else(|| CommandError::from("There is nothing to forgive"))?;

    let ranked_user = db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await?
        .ok_or_else(|| CommandError::from("Ranked user not found"))?;

    msg.channel_id.say(&ctx.http,
        format!("{}(#{}) was forgiven for '{}' and now only has {} bbps.",
            &ranked_user.friendly_name.unwrap(),
            &ranked_user.rank.unwrap(),
            &description,
            &ranked_user.points)).await?;

    Ok(())
}
fn parse_command_mention_and_description(msg: &Message, args: Args) -> Result<(String, String, String), CommandError> {
    let (issuer, target, args) = parse_command_mention_only(msg, args)?;
    Ok((issuer, target, args.rest().to_string()))
}

fn parse_command_mention_only(msg: &Message, mut args: Args) -> Result<(String, String, Args), CommandError> {
    let issuer = msg.author.mention().to_string();
    let target = args.single::<String>()?;

    if RE.is_match(&target) {
        Ok((issuer, target, args))
    } else if target.starts_with('@') {
        Err(CommandError::from(format!("It looks like you didn't mention {} correctly, FIX IT, PRESS TAB.", target)))
    } else {
        Err(CommandError::from("Idk what that means... thats a bbp for you."))
    }
}

fn print_and_return_err<T, E: std::fmt::Display>(err: E, message: &str) -> Result<T, E> {
    eprintln!("{}: {}", message, err);
    Err(err)
}

fn print_and_return_none<T>(message: &str) -> Option<T> {
    println!("{}", message);
    None
}
