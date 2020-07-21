use crate::{
    dispatching::{
        update_listeners, update_listeners::UpdateListener, DispatcherHandler,
        UpdateWithCx,
    },
    error_handlers::{ErrorHandler, LoggingErrorHandler},
    types::{
        CallbackQuery, ChosenInlineResult, InlineQuery, Message, Poll,
        PollAnswer, PreCheckoutQuery, ShippingQuery, UpdateKind,
    },
    utils::command::{BotCommand, ParseError},
    Bot,
};
use futures::StreamExt;
use serde::export::Formatter;
use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};
use tokio::sync::mpsc;

type Tx<Upd> = Option<mpsc::UnboundedSender<UpdateWithCx<Upd>>>;

#[macro_use]
mod macros {
    /// Pushes an update to a queue.
    macro_rules! send {
        ($bot:expr, $tx:expr, $update:expr, $variant:expr) => {
            send($bot, $tx, $update, stringify!($variant));
        };
    }
}

fn send<'a, Upd>(
    bot: &'a Bot,
    tx: &'a Tx<Upd>,
    update: Upd,
    variant: &'static str,
) where
    Upd: Debug,
{
    if let Some(tx) = tx {
        if let Err(error) = tx.send(UpdateWithCx { bot: bot.clone(), update }) {
            log::error!(
                "The RX part of the {} channel is closed, but an update is \
                 received.\nError:{}\n",
                variant,
                error
            );
        }
    }
}

/// A bot command that always fails.
#[derive(Debug, Copy, Clone)]
pub struct FailingBotCommand;

impl BotCommand for FailingBotCommand {
    fn descriptions() -> String {
        unimplemented!()
    }

    fn parse<N>(_s: &str, _bot_name: N) -> Result<Self, ParseError>
    where
        N: Into<String>,
    {
        Err(ParseError::Custom(Box::new(FailingBotCommandError)))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FailingBotCommandError;

impl Display for FailingBotCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FailingBotCommandError")
    }
}

impl Error for FailingBotCommandError {}

/// One dispatcher to rule them all.
///
/// See [the module-level documentation for the design
/// overview](crate::dispatching).
pub struct Dispatcher<Cmd = FailingBotCommand> {
    bot: Bot,

    messages_queue: Tx<Message>,
    edited_messages_queue: Tx<Message>,
    channel_posts_queue: Tx<Message>,
    edited_channel_posts_queue: Tx<Message>,
    inline_queries_queue: Tx<InlineQuery>,
    chosen_inline_results_queue: Tx<ChosenInlineResult>,
    callback_queries_queue: Tx<CallbackQuery>,
    shipping_queries_queue: Tx<ShippingQuery>,
    pre_checkout_queries_queue: Tx<PreCheckoutQuery>,
    polls_queue: Tx<Poll>,
    poll_answers_queue: Tx<PollAnswer>,

    commands_queue: Tx<(Message, Cmd)>,
    bot_name: Option<String>,
}

impl<Cmd> Dispatcher<Cmd> {
    /// Constructs a new dispatcher with the specified `bot`.
    #[must_use]
    pub fn new(bot: Bot) -> Self {
        Self {
            bot,
            messages_queue: None,
            edited_messages_queue: None,
            channel_posts_queue: None,
            edited_channel_posts_queue: None,
            inline_queries_queue: None,
            chosen_inline_results_queue: None,
            callback_queries_queue: None,
            shipping_queries_queue: None,
            pre_checkout_queries_queue: None,
            polls_queue: None,
            poll_answers_queue: None,
            commands_queue: None,
            bot_name: None,
        }
    }

    #[must_use]
    fn new_tx<H, Upd>(&self, h: H) -> Tx<Upd>
    where
        H: DispatcherHandler<Upd> + Send + 'static,
        Upd: Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let fut = h.handle(rx);
            fut.await;
        });
        Some(tx)
    }

    #[must_use]
    pub fn messages_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<Message> + 'static + Send,
    {
        self.messages_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn edited_messages_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<Message> + 'static + Send,
    {
        self.edited_messages_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn channel_posts_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<Message> + 'static + Send,
    {
        self.channel_posts_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn edited_channel_posts_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<Message> + 'static + Send,
    {
        self.edited_channel_posts_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn inline_queries_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<InlineQuery> + 'static + Send,
    {
        self.inline_queries_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn chosen_inline_results_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<ChosenInlineResult> + 'static + Send,
    {
        self.chosen_inline_results_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn callback_queries_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<CallbackQuery> + 'static + Send,
    {
        self.callback_queries_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn shipping_queries_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<ShippingQuery> + 'static + Send,
    {
        self.shipping_queries_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn pre_checkout_queries_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<PreCheckoutQuery> + 'static + Send,
    {
        self.pre_checkout_queries_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn polls_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<Poll> + 'static + Send,
    {
        self.polls_queue = self.new_tx(h);
        self
    }

    #[must_use]
    pub fn poll_answers_handler<H>(mut self, h: H) -> Self
    where
        H: DispatcherHandler<PollAnswer> + 'static + Send,
    {
        self.poll_answers_queue = self.new_tx(h);
        self
    }

    /// Sets a handler for commands.
    ///
    /// Commands will be passed into this handler if an incoming message
    /// contains text, and if this text is a valid command (according to the
    /// `Cmd` generic). In this case, the message will **NOT** be passed into a
    /// handler for messages, otherwise, it will be.
    #[must_use]
    pub fn commands_handler<H, S>(mut self, h: H, bot_name: S) -> Self
    where
        Cmd: BotCommand + 'static + Send,
        H: DispatcherHandler<(Message, Cmd)> + 'static + Send,
        S: Into<String>,
    {
        self.commands_queue = self.new_tx(h);
        self.bot_name = Some(bot_name.into());
        self
    }

    /// Starts your bot with the default parameters.
    ///
    /// The default parameters are a long polling update listener and log all
    /// errors produced by this listener).
    pub async fn dispatch(&self)
    where
        Cmd: BotCommand + 'static + Send + Debug,
    {
        self.dispatch_with_listener(
            update_listeners::polling_default(self.bot.clone()),
            LoggingErrorHandler::with_custom_text(
                "An error from the update listener",
            ),
        )
        .await;
    }

    /// Starts your bot with custom `update_listener` and
    /// `update_listener_error_handler`.
    pub async fn dispatch_with_listener<'a, UListener, ListenerE, Eh>(
        &'a self,
        update_listener: UListener,
        update_listener_error_handler: Arc<Eh>,
    ) where
        UListener: UpdateListener<ListenerE> + 'a,
        Eh: ErrorHandler<ListenerE> + 'a,
        ListenerE: Debug,
        Cmd: BotCommand + 'static + Send + Debug,
    {
        let update_listener = Box::pin(update_listener);

        update_listener
            .for_each(move |update| {
                let update_listener_error_handler =
                    Arc::clone(&update_listener_error_handler);

                async move {
                    log::trace!("Dispatcher received an update: {:?}", update);

                    let update = match update {
                        Ok(update) => update,
                        Err(error) => {
                            Arc::clone(&update_listener_error_handler)
                                .handle_error(error)
                                .await;
                            return;
                        }
                    };

                    match update.kind {
                        UpdateKind::Message(message) => {
                            if self.commands_queue.is_some() {
                                if let Some(text) = message.text() {
                                    if let Ok(cmd) = Cmd::parse(
                                        text,
                                        self.bot_name
                                            .as_ref() /* Because commands_queue is
                                             * Some */
                                            .unwrap(),
                                    ) {
                                        send!(
                                            &self.bot,
                                            &self.commands_queue,
                                            (message, cmd),
                                            commands_queue
                                        );
                                        return;
                                    }
                                }
                            }

                            send!(
                                &self.bot,
                                &self.messages_queue,
                                message,
                                UpdateKind::Message
                            );
                        }
                        UpdateKind::EditedMessage(message) => {
                            send!(
                                &self.bot,
                                &self.edited_messages_queue,
                                message,
                                UpdateKind::EditedMessage
                            );
                        }
                        UpdateKind::ChannelPost(post) => {
                            send!(
                                &self.bot,
                                &self.channel_posts_queue,
                                post,
                                UpdateKind::ChannelPost
                            );
                        }
                        UpdateKind::EditedChannelPost(post) => {
                            send!(
                                &self.bot,
                                &self.edited_channel_posts_queue,
                                post,
                                UpdateKind::EditedChannelPost
                            );
                        }
                        UpdateKind::InlineQuery(query) => {
                            send!(
                                &self.bot,
                                &self.inline_queries_queue,
                                query,
                                UpdateKind::InlineQuery
                            );
                        }
                        UpdateKind::ChosenInlineResult(result) => {
                            send!(
                                &self.bot,
                                &self.chosen_inline_results_queue,
                                result,
                                UpdateKind::ChosenInlineResult
                            );
                        }
                        UpdateKind::CallbackQuery(query) => {
                            send!(
                                &self.bot,
                                &self.callback_queries_queue,
                                query,
                                UpdateKind::CallbackQuer
                            );
                        }
                        UpdateKind::ShippingQuery(query) => {
                            send!(
                                &self.bot,
                                &self.shipping_queries_queue,
                                query,
                                UpdateKind::ShippingQuery
                            );
                        }
                        UpdateKind::PreCheckoutQuery(query) => {
                            send!(
                                &self.bot,
                                &self.pre_checkout_queries_queue,
                                query,
                                UpdateKind::PreCheckoutQuery
                            );
                        }
                        UpdateKind::Poll(poll) => {
                            send!(
                                &self.bot,
                                &self.polls_queue,
                                poll,
                                UpdateKind::Poll
                            );
                        }
                        UpdateKind::PollAnswer(answer) => {
                            send!(
                                &self.bot,
                                &self.poll_answers_queue,
                                answer,
                                UpdateKind::PollAnswer
                            );
                        }
                    }
                }
            })
            .await
    }
}
