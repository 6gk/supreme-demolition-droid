use maplit::hashmap;

use super::*;

pub async fn message(ctx: client::Context, msg: Message) -> Result<()> {
    let config = ctx.data.read().await.get::<Config>().unwrap().clone();
    if msg.author.bot {
        return Ok(());
    }
    if msg.channel_id == config.channel_showcase {
        handle_showcase_post(ctx, msg)
            .await
            .context("Failed to handle showcase post")
    } else if msg.channel_id == config.channel_feedback {
        handle_feedback_post(ctx, msg)
            .await
            .context("Failed to handle feedback post")
    } else {
        Ok(())
    }
}

async fn handle_showcase_post(ctx: client::Context, msg: Message) -> Result<()> {
    if !msg.attachments.is_empty() || !msg.embeds.is_empty() {
        if let Some(attachment) = msg.attachments.first() {
            let data = ctx.data.read().await;
            let db = data.get::<Db>().unwrap().clone();
            msg.react(&ctx, ReactionType::Unicode("❤️".to_string()))
                .await
                .context("Error reacting to showcase submission with ❤️")?;

            db.update_fetch(
                msg.author.id,
                hashmap! {"image".to_string() => attachment.url.to_string() },
            )
            .await?;
        }
    } else {
        msg.delete(&ctx)
            .await
            .context("Failed to delete invalid showcase submission")?;
        msg.author.direct_message(&ctx, |f| {
                f.content(indoc!("
                    Your showcase submission was detected to be invalid. If you wanna comment on a rice, use the #ricing-theming channel.
                    If this is a mistake, contact the moderators or open an issue on https://github.com/unixporn/trup
                "))
            }).await.context("Failed to send DM about invalid showcase submission")?;
    }
    Ok(())
}

async fn handle_feedback_post(ctx: client::Context, msg: Message) -> Result<()> {
    msg.react(&ctx, ReactionType::Unicode("👍".to_string()))
        .await
        .context("Error reacting to feedback submission with 👍")?;
    msg.react(&ctx, ReactionType::Unicode("👎".to_string()))
        .await
        .context("Error reacting to feedback submission with 👍")?;

    // retrieve the last keep-at-bottom message the bot wrote
    let recent_messages = msg.channel_id.messages(&ctx, |m| m.before(&msg)).await?;

    let last_bottom_pin_msg = recent_messages.iter().find(|m| {
        m.author.bot
            && m.embeds
                .iter()
                .any(|e| e.title == Some("CONTRIBUTING.md".to_string()))
    });
    if let Some(bottom_pin_msg) = last_bottom_pin_msg {
        bottom_pin_msg.delete(&ctx).await?;
    }
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("CONTRIBUTING.md").color(0xb8bb26);
            e.description(indoc::indoc!(
                "Before posting, please make sure to check if your idea is a **repetitive topic**. (Listed in pins)
                Note that we have added a consequence for failure. The inability to delete repetitive feedback will result in an 'unsatisfactory' mark on your official testing record, followed by death. Good luck!"
            ))
        })
    }).await?;
    Ok(())
}
