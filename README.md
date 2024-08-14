# Shreck Deck

A crate for making custom TTS decks.

## How To Use

Firstly, create a type for your cards. It must implement the `GetCardInfo` trait.

```rust
enum MyCard {
  Witch,
  Mechanic,
  JaywaltzTheLiar,
}

impl GetCardInfo for MyCard {
    fn get_name(&self) -> &str {
        match self {
          Self::Witch => "Witch",
          Self::Mechanic => "Mechanic",
          Self::JaywaltzTheLiar => "Jaywaltz",
        }
    }

    fn get_front_image(&self) -> Result<String, shrek_deck::CardError> {
        /* get the url from the card's name */
    }

    fn get_back_image(&self) -> Result<String, shrek_deck::CardError> {
        /* get the url from the card's name */
    }

    fn get_card_shape(&self) -> Result<CardShape, shrek_deck::CardError> {
        Ok(CardShape::RoundedRectangle)
    }

    fn parse(string: &str) -> Result<Self, shrek_deck::parser::ParseError> {
        /* how to turn a string into a card. you usually want this to turn the name of the card into the card data structure */
    }
}
```

The crate offers a parser for files.

```rust
let cards = parse_file::<BloodlessCard>("some/input/path.txt").unwrap();
```

You can then create a save file with that file, serialize it with serde, and save it wherever you want.

```rust
let save = SaveState::new_with_deck(cards).unwrap;
let contents = serde_json::to_string_pretty(&save).unwrap();
std::fs::write("some/relative/path.json", contents).unwrap();
```

Or save directly to the TTS objects dir. This requires you to provide an image as its icon.

```rust
write_to_tts_dir("some/relative/path.json", contents, include_bytes!("blood.png")).unwrap();
```
