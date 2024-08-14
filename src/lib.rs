#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #[cfg(feature = "parser")]
pub mod parser;
pub mod tts;

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tts::{CardShape, CustomDeckState};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardError {
    CardDoesntExist {
        card_name: String,
    },
    BackImageFileError {
        card_name: String,
        image_url: String,
    },
    FrontImageNotFound {
        card_name: String,
        image_url: String,
    },
    Custom {
        message: String,
    },
}

impl Display for CardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardDoesntExist { card_name } => write!(f, "Card doesn't exist: {card_name}"),
            Self::BackImageFileError {
                card_name,
                image_url,
            } => write!(
                f,
                "Couldn't find the file for {card_name}'s back: {image_url}"
            ),
            Self::FrontImageNotFound {
                card_name,
                image_url,
            } => write!(
                f,
                "Couldn't find the file for {card_name}'s front: {image_url}"
            ),
            Self::Custom { message } => write!(f, "{message}"),
        }
    }
}

impl CardError {
    #[must_use]
    pub const fn custom(message: String) -> Self {
        Self::Custom { message }
    }
}

/// Trait for all things that are cards
pub trait GetCardInfo: Sized {
    /// The card's name
    fn get_name(&self) -> &str;
    /// The card's front image URL
    /// # Errors
    /// Whenever you decide
    fn get_front_image(&self) -> Result<String, CardError>;
    /// The card's back image URL
    /// # Errors
    /// Whenever you decide
    fn get_back_image(&self) -> Result<String, CardError>;
    /// The card shape
    /// # Errors
    /// Whenever you decide
    fn get_card_shape(&self) -> Result<CardShape, CardError>;
    /// Turns a String into a card.
    /// # Errors
    /// Whenever you decide
    // #[cfg(feature="parser")]
    fn parse(string: &str) -> Result<Self, parser::ParseError>;
}

#[derive(Clone)]
pub struct CardEntry<T: GetCardInfo + Clone> {
    pub card: T,
    pub amount: i64,
}

impl<T: GetCardInfo + Clone> CardEntry<T> {
    /// # Errors
    /// Whenever any of the `GetCardInfo` implementations in the supplied type error.
    pub fn get_custom_deck_state(&self) -> Result<CustomDeckState, CardError> {
        Ok(CustomDeckState {
            name: self.card.get_name().to_owned(),
            face_url: self.card.get_front_image()?,
            back_url: self.card.get_back_image()?,
            num_width: Some(1),
            num_height: Some(1),
            back_is_hidden: true,
            unique_back: false,
            r#type: self.card.get_card_shape()?.into(),
        })
    }
}

fn generate_guid() -> String {
    Uuid::new_v4().to_string()
}
