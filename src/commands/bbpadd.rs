use std::future::Future;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

use crate::PostgresServiceContainer;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^<@\d+>$").unwrap();
}

#[command]
#[bucket = "complicated"]
#[sub_commands(bbpadd)]
async fn bbp(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[bucket = "complicated"]
#[sub_commands(gbpadd)]
async fn gbp(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn bbpadd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    add(ctx, msg, args, move |author, target, description| {

        let author = author.to_owned();
        let target = target.to_owned();
        let description = description.to_owned();
        async move {
            let data_read = ctx.data.read().await;
            let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
            let db = db_lock.lock().await; 

            match db.get_user_by_discord_mention(&author).await {
                Ok(Some(author_user)) => {
                    match db.get_user_by_discord_mention(&target).await {
                        Ok(Some(target_user)) => {
                            match db.add_bbp_to_user(&target_user, &author_user, &description).await {
                                Ok(_) => {
                                    match db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await {
                                        Ok(Some(ranked_user)) => {
                                            if let Err(why) = msg.channel_id.say(&ctx.http, 
                                                format!("{} now has {} bbps and is the #{} baddest boi.", &ranked_user.friendly_name.unwrap(), &ranked_user.points, &ranked_user.rank.unwrap())).await {
                                                eprintln!("Error sending message: {:?}", why);
                                            }
                                        },
                                        Ok(_) => println!("Target user not found: {}", target),
                                        Err(err) => eprintln!("Error getting target user rank: {}", err),
                                    }
                                    println!("BBP added successfully: author - {}, target - {}, description - '{}'", author, target, description)
                                }
                                Err(err) => eprintln!("Error adding BBP: {}", err),
                            }
                        },
                        Ok(None) => println!("Target user not found: {}", target),
                        Err(err) => eprintln!("Error getting target user: {}", err),
                    }
                },
                Ok(None) => println!("Author user not found: {}", author),
                Err(err) => eprintln!("Error getting author user: {}\n {:?}", err, author),
            }
        }
    }).await
}

#[command]
#[bucket = "complicated"]
#[aliases("add")]
async fn gbpadd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    add(ctx, msg, args, move |author, target, description| {
        let author = author.to_owned();
        let target = target.to_owned();
        let description = description.to_owned();

        async move {
            let data_read = ctx.data.read().await;
            let db_lock = data_read.get::<PostgresServiceContainer>().expect("Expected PostgresService in TypeMap.");
            let db = db_lock.lock().await; 

            match db.get_user_by_discord_mention(&author).await {
                Ok(Some(author_user)) => {
                    match db.get_user_by_discord_mention(&target).await {
                        Ok(Some(target_user)) => {
                            match db.add_gbp_to_user(&target_user, &author_user, &description).await {
                                Ok(_) => {
                                    match db.get_user_by_discord_mention_with_rank(&target_user.discord_mention.unwrap()).await {
                                        Ok(Some(ranked_user)) => {
                                            if let Err(why) = msg.channel_id.say(&ctx.http, 
                                                format!("{} now has {} bbps and is the #{} baddest boi.", &ranked_user.friendly_name.unwrap(), &ranked_user.points, &ranked_user.rank.unwrap())).await {
                                                eprintln!("Error sending message: {:?}", why);
                                            }
                                        },
                                        Ok(_) => println!("Target user not found: {}", target),
                                        Err(err) => eprintln!("Error getting target user rank: {}", err),
                                    }
                                    println!("GBP added successfully: author - {}, target - {}, description - '{}'", author, target, description)
                                },
                                Err(err) => eprintln!("Error adding BBP: {}", err),
                            }
                        },
                        Ok(None) => println!("Target user not found: {}", target),
                        Err(err) => eprintln!("Error getting target user: {}", err),
                    }
                },
                Ok(None) => println!("Author user not found: {}", author),
                Err(err) => eprintln!("Error getting author user: {}", err),
            }
        }
    }).await
}

async fn add<F, Fut>(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
    db_call: F
) -> CommandResult 
where
    F: Fn(&str, &str, &str) -> Fut,
    Fut: Future<Output = ()>,
{
    let author = &msg.author.mention().to_string();
    let target = args.single::<String>()?;
    let description = args.rest();

    if RE.is_match(&target) {
        if description.is_empty() {            
            msg.channel_id.say(&ctx.http, "Write a description.").await?;
            return Ok(());
        } 
    } else if target.starts_with("@") {
        msg.channel_id.say(&ctx.http, format!("It looks like you didn't mention {} correctly, FIX IT, PRESS TAB.", target)).await?;
        return Ok(());
    } else {
        msg.channel_id.say(&ctx.http, "Idk what that means... thats a bbp for you.").await?;
        return Ok(());
    }

    db_call(&author, &target, &description).await;

    Ok(())
}