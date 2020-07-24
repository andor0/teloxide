use serde::Serialize;

use crate::{
    net,
    requests::{Request, ResponseResult},
    types::{ChatId, InlineKeyboardMarkup, Message},
    Bot,
};

/// Use this method to send an animated emoji that will display a random value.
///
/// [The official docs](https://core.telegram.org/bots/api#senddice).
#[serde_with_macros::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct SendDice {
    #[serde(skip_serializing)]
    bot: Bot,
    chat_id: ChatId,
    emoji: Option<String>,
    disable_notification: Option<bool>,
    reply_to_message_id: Option<i32>,
    reply_markup: Option<InlineKeyboardMarkup>,
}

#[async_trait::async_trait]
impl Request for SendDice {
    type Output = Message;

    async fn send(&self) -> ResponseResult<Message> {
        net::request_json(
            self.bot.client(),
            self.bot.token(),
            "sendDice",
            &self,
        )
        .await
    }
}

impl SendDice {
    pub(crate) fn new<T>(bot: Bot, chat_id: T) -> Self
    where
        T: Into<ChatId>,
    {
        Self {
            bot,
            chat_id: chat_id.into(),
            emoji: None,
            disable_notification: None,
            reply_to_message_id: None,
            reply_markup: None,
        }
    }
    /// Unique identifier for the target chat or username of the target channel
    /// (in the format @channelusername)
    pub fn chat_id<T>(mut self, val: T) -> Self
    where
        T: Into<ChatId>,
    {
        self.chat_id = val.into();
        self
    }

    /// Emoji on which the dice throw animation is based. Currently, must be
    /// one of “🎲”, “🎯”, or “🏀”. Dice can have values 1-6 for “🎲” and “🎯”,
    /// and values 1-5 for “🏀”. Defaults to “🎲”
    pub fn emoji<T>(mut self, val: T) -> Self
    where
        T: Into<String>,
    {
        self.emoji = Some(val.into());
        self
    }

    /// Sends the message silently. Users will receive a notification with no
    /// sound.
    pub fn disable_notification(mut self, val: bool) -> Self {
        self.disable_notification = Some(val);
        self
    }

    /// If the message is a reply, ID of the original message.
    pub fn reply_to_message_id(mut self, val: i32) -> Self {
        self.reply_to_message_id = Some(val);
        self
    }

    /// A JSON-serialized object for an [inline keyboard]. If empty, one `Play
    /// game_title` button will be shown. If not empty, the first button must
    /// launch the game.
    ///
    /// [inline keyboard]: https://core.telegram.org/bots#inline-keyboards-and-on-the-fly-updating
    pub fn reply_markup(mut self, val: InlineKeyboardMarkup) -> Self {
        self.reply_markup = Some(val);
        self
    }
}
