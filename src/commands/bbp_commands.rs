use log::{error};
use crate::{Context, Error};
use crate::dataaccess::postgres_service;
use std::fmt::Write;

#[poise::command(slash_command, rename = "bbp", user_cooldown = 30)]
pub async fn bbp_add_command(
    ctx: Context<'_>,
    target: poise::serenity_prelude::User,
    description: String
) -> Result<(), Error> {
    let issuer = ctx.author().id.get() as i64;
    let target = target.id.get() as i64;
    let db = ctx.data().db.lock().await;

    let issuing_user = match db.get_user_by_discord_id(issuer).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Issuing user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching issuing user: {}", e);
            ctx.say("Error fetching issuing user").await?;
            return Ok(());
        }
    };

    let target_user = match db.get_user_by_discord_id(target).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Target user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching target user: {}", e);
            ctx.say("Error fetching target user").await?;
            return Ok(());
        }
    };

    db.add_bbp_to_user(&target_user, &issuing_user, &description).await?;
    
    let ranked_user = match db.get_user_by_discord_id_with_rank(target).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Ranked user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error adding BBP: {}", e);
            ctx.say("Error adding BBP").await?;
            return Ok(());
        }
    };
    
    let msg = format!(
        "{} has given {} a bbp.\n\n{}\n\n{}(#{}) now has {} bbps.",
        issuing_user.friendly_name.unwrap(),
        target_user.discord_mention.unwrap(),
        description,
        ranked_user.friendly_name.unwrap(),
        ranked_user.rank.unwrap(),
        ranked_user.points
    );    
    
    ctx.say(msg).await?;

    Ok(())
}

#[poise::command(slash_command, rename = "gbp", user_cooldown = 30)]
pub async fn gbp_add_command(
    ctx: Context<'_>,
    target: poise::serenity_prelude::User,
    description: String
) -> Result<(), Error> {
    let issuer = ctx.author().id.get() as i64;
    let target = target.id.get() as i64;

    if issuer == target {
        let ranked_user = match add_bbp(&ctx, target.clone(), target, "Attempting to give themselves a GBP 😡".to_string()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                let msg = "Ranked user not found";
                error!("{}", msg);
                ctx.say(msg).await?;
                return Ok(());
            }
            Err(e) => {
                error!("Error adding GBP: {}", e);
                ctx.say("Error adding GBP").await?;
                return Ok(());
            }
        };

        ctx.say(format!("😡 trying to give yourself a gbp? That's a bbp for you. {}(#{}) now has {} bbps.",
            &ranked_user.friendly_name.unwrap(),
            &ranked_user.rank.unwrap(),
            &ranked_user.points)).await?;
    } else {
        match add_gbp(&ctx, issuer, target, &description).await {
            Ok((issuing_user, target_user, ranked_user)) => {
                let msg = format!(
                    "{} has given {} a bbp.\n\n{}\n\n{}(#{}) now has {} bbps.",
                    issuing_user.friendly_name.unwrap(),
                    target_user.discord_mention.unwrap(),
                    description,
                    ranked_user.friendly_name.unwrap(),
                    ranked_user.rank.unwrap(),
                    ranked_user.points
                );
                ctx.say(msg).await?;
            },
            Err(e) => {
                error!("Error adding GBP: {}", e);
                ctx.say("Error adding GBP").await?;
                return Ok(());
            }
        };
    }

    Ok(())
}

#[poise::command(slash_command, rename = "forgive")]
pub async fn bbp_forgive_command(
    ctx: Context<'_>,
    target: poise::serenity_prelude::User
) -> Result<(), Error> {
    let issuer = ctx.author().id.get() as i64;
    let target = target.id.get() as i64;
    let db = ctx.data().db.lock().await;

    let issuing_user = match db.get_user_by_discord_id(issuer).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Issuing user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching issuing user: {}", e);
            ctx.say("Error fetching issuing user").await?;
            return Ok(());
        }
    };

    let target_user = match db.get_user_by_discord_id(target).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Target user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching target user: {}", e);
            ctx.say("Error fetching target user").await?;
            return Ok(());
        }
    };

    let description = match db.forgive_user(&target_user, &issuing_user).await {
        Ok(Some(desc)) => desc,
        Ok(None) => {
            let msg = "There is nothing to forgive";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error forgiving user: {}", e);
            ctx.say("Error forgiving user").await?;
            return Ok(());
        }
    };

    let ranked_user = match db.get_user_by_discord_id_with_rank(target).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Ranked user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching ranked user: {}", e);
            ctx.say("Error fetching ranked user").await?;
            return Ok(());
        }
    };

    ctx.say(format!("{}(#{}) was forgiven for '{}' and now only has {} bbps.",
        &ranked_user.friendly_name.unwrap(),
        &ranked_user.rank.unwrap(),
        &description,
        &ranked_user.points)).await?;

    Ok(())
}

#[poise::command(slash_command, rename = "add-user", owners_only)]
pub async fn add_user_command(
    ctx: Context<'_>,
    target: poise::serenity_prelude::User,
    friendly_name: String,
) -> Result<(), Error> {
    let user_id = target.id.get() as i64;
    let user_name = target.name.clone();
    let db = ctx.data().db.lock().await;

    // Check if the user already exists
    if let Ok(Some(_)) = db.get_user_by_discord_id(user_id).await {
        let msg = "User already exists";
        error!("{}", msg);
        ctx.say(msg).await?;
        return Ok(());
    }

    let added_user = match db.add_user(user_id, &user_name, &friendly_name).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Ranked user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error adding user: {}", e);
            ctx.say("Error adding user").await?;
            return Ok(());
        }
    };
    
    ctx.say(format!("Added user '{}'({}/{})",
        &added_user.friendly_name.unwrap(),
        &added_user.discord_username.unwrap(),
        &added_user.user_id)).await?;

    Ok(())
}

#[poise::command(slash_command, rename = "leaderboard", user_cooldown = 30)]
pub async fn leaderboard_command(ctx: Context<'_>) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;

    let leaderboard = match db.get_leaderboard().await {
        Ok(users) => users,
        Err(e) => {
            error!("Error fetching leaderboard: {}", e);
            ctx.say("Error fetching leaderboard").await?;
            return Ok(());
        }
    };

    let mut response = String::new();
    for user in leaderboard {
        let _ = writeln!(
            response,
            "{}. {} ({} points, {} bbps given, {} gbps given)",
            user.rank,
            user.friendly_name.unwrap(),
            user.points,
            user.bbps_issued,
            user.gbps_issued
        );
    }

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "history", user_cooldown = 30)]
pub async fn history_command(
    ctx: Context<'_>,
    user: Option<poise::serenity_prelude::User>
) -> Result<(), Error> {
    let target_user = match user {
        Some(u) => u,
        None => ctx.author().clone(),
    };

    let db = ctx.data().db.lock().await;

    let user_id = target_user.id.get() as i64;

    // Fetch the target user information
    let target_user_info = match db.get_user_by_discord_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let msg = "Target user not found";
            error!("{}", msg);
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            error!("Error fetching target user: {}", e);
            ctx.say("Error fetching target user").await?;
            return Ok(());
        }
    };

    let history = match db.get_user_history(user_id).await {
        Ok(records) => records,
        Err(e) => {
            error!("Error fetching user history: {}", e);
            ctx.say("Error fetching user history").await?;
            return Ok(());
        }
    };

    let mut response = format!("History for {}\n\n", target_user_info.friendly_name.unwrap_or("Unknown".to_string()));
    for record in history {
        let date = record.timestamp.format("%Y-%m-%d").to_string();
        let _ = writeln!(
            response,
            "{} -> {} ({})",
            record.issuer_friendly_name,
            record.description,
            date
        );
    }

    if response.is_empty() {
        response = "No history found".to_string();
    }
    ctx.say(response).await?;
    Ok(())
}
async fn add_bbp(ctx: &Context<'_>, issuer: i64, target: i64, description: String) -> Result<Option<postgres_service::User>, Box<dyn std::error::Error + Send + Sync>> {
    let db = ctx.data().db.lock().await;

    let issuing_user = match db.get_user_by_discord_id(issuer).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Issuing user not found".into()),
        Err(e) => return Err(e.into()),
    };

    let target_user = match db.get_user_by_discord_id(target).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Target user not found".into()),
        Err(e) => return Err(e.into()),
    };

    db.add_bbp_to_user(&target_user, &issuing_user, &description).await?;

    let ranked_user = match db.get_user_by_discord_id_with_rank(target).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Ranked user not found".into()),
        Err(e) => return Err(e.into()),
    };

    Ok(Some(ranked_user))
}

async fn add_gbp(
    ctx: &Context<'_>, 
    issuer: i64, 
    target: i64, 
    description: &str
) -> Result<(postgres_service::User, postgres_service::User, postgres_service::User), Box<dyn std::error::Error + Send + Sync>> {
    let db = ctx.data().db.lock().await;

    let issuing_user = match db.get_user_by_discord_id(issuer).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Issuing user not found".into()),
        Err(e) => return Err(e.into()),
    };

    let target_user = match db.get_user_by_discord_id(target).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Target user not found".into()),
        Err(e) => return Err(e.into()),
    };

    db.add_gbp_to_user(&target_user, &issuing_user, &description).await?;

    let ranked_user = match db.get_user_by_discord_id_with_rank(target).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Ranked user not found".into()),
        Err(e) => return Err(e.into()),
    };

    Ok((issuing_user, target_user, ranked_user))
}