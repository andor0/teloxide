use crate::{
    dispatching::{dialogue::DialogueStage, UpdateWithCx},
    requests::ResponseResult,
    types::Message,
};
use futures::future::BoxFuture;

/// Represents a transition function of a dialogue FSM.
pub trait Transition: Sized {
    type Aux;

    /// Turns itself into another state, depending on the input message.
    ///
    /// `aux` will be passed to each subtransition function.
    fn react(self, cx: TransitionIn, aux: Self::Aux) -> BoxFuture<'static, TransitionOut<Self>>;
}

/// Like [`Transition`], but from `StateN` -> `Dialogue`.
///
/// [`Transition`]: crate::dispatching::dialogue::Transition
pub trait Subtransition
where
    Self::Dialogue: Transition<Aux = Self::Aux>,
{
    type Aux;
    type Dialogue;

    /// Turns itself into another state, depending on the input message.
    ///
    /// `aux` is something that is provided by the call side, for example,
    /// message's text.
    fn react(
        self,
        cx: TransitionIn,
        aux: Self::Aux,
    ) -> BoxFuture<'static, TransitionOut<Self::Dialogue>>;
}

/// A type returned from a FSM subtransition function.
///
/// Now it is used only inside `#[teloxide(subtransition)]` for type inference.
pub trait SubtransitionOutputType {
    type Output;
}

impl<D> SubtransitionOutputType for TransitionOut<D> {
    type Output = D;
}

/// An input passed into a FSM (sub)transition function.
pub type TransitionIn = UpdateWithCx<Message>;

/// A type returned from a FSM (sub)transition function.
pub type TransitionOut<D> = ResponseResult<DialogueStage<D>>;
