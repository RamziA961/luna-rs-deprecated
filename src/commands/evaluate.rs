use crate::{
    config::{Context, Error},
    utils::calculator,
};

/// Quickly evaluate a math expression.
#[poise::command(slash_command)]
pub async fn eval(context: Context<'_>, expr: Option<String>) -> Result<(), Error> {
    context
        .say(calculator::eval(expr.unwrap().as_str()).unwrap().as_str())
        .await?;

    Ok(())
}
