use serde::{Deserialize, Serialize};

#[serde_with_macros::skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Dice {
    pub emoji: String,
    pub value: i32,
}
