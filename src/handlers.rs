use teloxide::{
    payloads::AnswerInlineQuery,
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        Me, ParseMode,
    },
};

pub async fn handle_inline(bot: Bot, q: InlineQuery, me: Me) -> ResponseResult<()> {
    if q.query.is_empty() {
        return Ok(());
    }

    Ok(())
}
