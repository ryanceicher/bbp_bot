use serenity::framework::standard::macros::group;
use crate::commands::bbpadd::BBP_COMMAND;
use crate::commands::bbpadd::GBP_COMMAND;

pub mod bbpadd;
pub mod commands;

#[group]
#[commands(bbp, gbp)]
struct General;