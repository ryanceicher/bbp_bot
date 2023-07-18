use serenity::{framework::standard::{macros::command, Args, CommandResult}, prelude::Context, model::prelude::Message};
use std::fmt::Write;

#[command]
#[bucket = "complicated"]
#[aliases("commands")]
async fn commands_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut commands_response = "Available !bbp commands:\n".to_string();

    writeln!(commands_response, "- !bbp add [@mention] [description]")?;
    writeln!(commands_response, "   - Adds the a bbp for the mentioned with the provided description.")?;
    writeln!(commands_response, "- !gbp add [@mention] [description]")?;
    writeln!(commands_response, "   - Adds the a gbp for the mentioned with the provided description.")?;
    writeln!(commands_response, "- !bbp add [@mention]")?;
    writeln!(commands_response, "   - Forgives the mentioned user for their last transgression against you.")?;
    
    if let Err(why) = msg.channel_id.say(&ctx.http, &commands_response).await {
        eprintln!("Error sending message: {:?}", why);
    }
    
    Ok(())
}