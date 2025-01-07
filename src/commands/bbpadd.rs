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
    if let Ok((issuer, target, description)) = parse_command_mention_and_description(msg, args) {
        if let Ok(Some(ranked_user)) = add_bbp(ctx, issuer, target, description).await {
            if let Err(why) = msg.channel_id.say(&ctx.http, 
                format!("{}(#{}) now has {} bbps.", 
                    &ranked_user.friendly_name.unwrap(), 
                    &ranked_user.rank.unwrap(), 
                    &ranked_user.points)).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn bbpadd_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Ok((issuer, target, description)) = parse_command_mention_and_description(msg, args) {
        if let Ok(Some(ranked_user)) = add_bbp(ctx, issuer, target, description).await {
            if let Err(why) = msg.channel_id.say(&ctx.http, 
                format!("{}(#{}) now has {} bbps.", 
                    &ranked_user.friendly_name.unwrap(), 
                    &ranked_user.rank.unwrap(), 
                    &ranked_user.points)).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn gbpadd_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Ok((issuer, target, description)) = parse_command_mention_and_description(msg, args) {
        // If someone tries to give themselves a gbp, they get a bbp instead.
        if issuer == target {
            if let Ok(Some(ranked_user)) = add_bbp(ctx, target.clone(), target, "Attempting to give themselves a GBP 😡".to_string()).await {
                if let Err(why) = msg.channel_id.say(&ctx.http, 
                    format!("😡 trying to give yourself a gbp? Thats a bbp for you. {}(#{}) now has {} bbps.", 
                        &ranked_user.friendly_name.unwrap(), 
                        &ranked_user.rank.unwrap(), 
                        &ranked_user.points)).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
        } else {
            // Add the gbp normally
            if let Ok(Some(ranked_user)) = add_gbp(ctx, issuer, target, description).await {
                if let Err(why) = msg.channel_id.say(&ctx.http, 
                    format!("{}(#{}) now has {} bbps.", 
                        &ranked_user.friendly_name.unwrap(), 
                        &ranked_user.rank.unwrap(), 
                        &ranked_user.points)).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
        }
    }

    Ok(())
}

async fn add_bbp(ctx: &Context, issuer: String, target: String, description: String) -> Result<Option<postgres_service::User>, Box<dyn std::error::Error + Send + Sync>> {
    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    match db.get_user_by_discord_mention(&issuer).await {
        Ok(Some(issuing_user)) => match db.get_user_by_discord_mention(&target).await {
            Ok(Some(target_user)) => match db.add_bbp_to_user(&target_user, &issuing_user, &description).await {
                Ok(_) => match db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await {
                    Ok(Some(ranked_user)) => Ok(Some(ranked_user)),
                    Ok(_) => Ok(print_and_return_none("Ranked user not found")),
                    Err(err) => print_and_return_err(err, "Error getting ranked target user"),
                }
                Err(err) => print_and_return_err(err, "Error adding BBP: {}")
            },
            Ok(None) => Ok(print_and_return_none("Target user not found")),
            Err(err) => print_and_return_err(err, "Error getting target user: {}"),
        },
        Ok(None) => Ok(print_and_return_none("Issuing user not found")),
        Err(err) => print_and_return_err(err, "Error getting issuing user: {}"),
    }
}

async fn add_gbp(ctx: &Context, issuer: String, target: String, description: String) -> Result<Option<postgres_service::User>, Box<dyn std::error::Error + Send + Sync>> {
    let data_read = ctx.data.read().await;
    let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
    let db = db_lock.lock().await;

    match db.get_user_by_discord_mention(&issuer).await {
        Ok(Some(issuing_user)) => match db.get_user_by_discord_mention(&target).await {
            Ok(Some(target_user)) => match db.add_gbp_to_user(&target_user, &issuing_user, &description).await {
                Ok(_) => match db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await {
                    Ok(Some(ranked_user)) => Ok(Some(ranked_user)),
                    Ok(_) => Ok(print_and_return_none("Ranked user not found")),
                    Err(err) => print_and_return_err(err, "Error getting ranked target user"),
                }
                Err(err) => print_and_return_err(err, "Error adding BBP: {}")
            },
            Ok(None) => Ok(print_and_return_none("Target user not found")),
            Err(err) => print_and_return_err(err, "Error getting target user: {}"),
        },
        Ok(None) => Ok(print_and_return_none("Issuing user not found")),
        Err(err) => print_and_return_err(err, "Error getting issuing user: {}"),
    }
}

#[command]
#[bucket = "complicated"]
#[aliases("forgive")]
async fn forgive(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match parse_command_mention_only(msg, args) {
        Ok((issuer, target, _)) => {
            let data_read = ctx.data.read().await;
            let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
            let db = db_lock.lock().await;

            match db.get_user_by_discord_mention(&issuer).await {
                Ok(Some(issuing_user)) => match db.get_user_by_discord_mention(&target).await {
                    Ok(Some(target_user)) => match db.forgive_user(&target_user, &issuing_user).await {
                        Ok(Some(description)) => match db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await {
                            Ok(Some(ranked_user)) => 
                                if let Err(why) = msg.channel_id.say(&ctx.http, 
                                    format!("{}(#{}) was forgiven for '{}' and now only has {} bbps.", 
                                        &ranked_user.friendly_name.unwrap(), 
                                        &ranked_user.rank.unwrap(), 
                                        &description, 
                                        &ranked_user.points)).await {
                                    eprintln!("Error sending message: {:?}", why);
                            },
                            Ok(_) => println!("Target user not found: {}", target),
                            Err(err) => eprintln!("Error getting target user rank: {}", err),
                        },
                        Ok(None) => if let Err(why) = msg.channel_id.say(&ctx.http, "There is nothing to forgive.").await {
                            eprintln!("Error sending message: {:?}", why);
                        },
                        Err(err) => eprintln!("Error adding forgiveness for BBP: {}", err),
                    },
                    Ok(None) => println!("Target user not found: {}", target),
                    Err(err) => eprintln!("Error getting target user: {}", err),
                },
                Ok(None) => println!("Issuing user not found: {}", issuer),
                Err(err) => eprintln!("Error getting issuing user: {}\n {:?}", err, issuer),
            }

            Ok(())
        },
        Err(err) => { 
            msg.channel_id.say(&ctx.http, "TODO WHAT IS WRONG HERE").await?;
            Err(err)
        }
    }
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
