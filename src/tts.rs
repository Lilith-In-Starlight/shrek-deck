#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
use std::{
    collections::HashMap,
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{generate_guid, CardEntry, CardError, GetCardInfo};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// Partial implementation of Tabletop Simulator's `SaveState`. Fields may be missing, both because the [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated, and because this implementation only contains the minimum necessary to support a deck of custom cards.
pub struct SaveState {
    save_name: String,
    date: String,
    version_number: String,
    game_mode: String,
    game_type: String,
    game_complexity: String,
    tags: Vec<String>,
    gravity: f64,
    play_area: f64,
    table: String,
    sky: String,
    note: String,
    tab_states: HashMap<String, String>,
    lua_script: String,
    lua_script_state: String,
    #[serde(rename = "XmlUI")]
    xml_ui: String,
    object_states: Vec<ObjectState>,
}

impl SaveState {
    /// Takes a vector of `CardEntry` and provides a `SaveState` for that deck. All saved objects in Tabletop Simulator are `SaveStates`.
    /// # Errors
    /// Under any situation that the `GetCardInfo` implementations of the provided type would error.
    pub fn new_with_deck<T: GetCardInfo + Clone>(
        deck: Vec<CardEntry<T>>,
    ) -> Result<Self, CardError> {
        let (deck_ids, custom_deck, contained_objects) = generate_deck_data(deck)?;
        let (deck_ids, contained_objects) = (Some(deck_ids), Some(contained_objects));
        let object_state = ObjectState {
            guid: generate_guid(),
            name: "Deck".to_string(),
            transform: TransformState {
                rot_y: 180.0,
                ..Default::default()
            },
            nickname: String::new(),
            description: String::new(),
            gm_notes: String::new(),
            alt_look_angle: Vector3::default(),
            color_difuse: ColourState {
                r: 0.713_235_259,
                g: 0.713_235_259,
                b: 0.713_235_259,
            },
            layout_group_sort_index: 0,
            value: 0,
            locked: false,
            grid: true,
            snap: true,
            ignore_fow: false,
            measure_movement: false,
            drag_selectable: true,
            autoraise: true,
            sticky: true,
            tooltip: true,
            grid_projection: false,
            hide_when_face_down: true,
            hands: false,
            card_id: None,
            sideways_card: false,
            deck_ids,
            custom_deck,
            lua_script: String::new(),
            lua_script_state: String::new(),
            xml_ui: String::new(),
            contained_objects,
        };
        let object_states = vec![object_state];
        Ok(Self {
            save_name: String::new(),
            date: String::new(),
            version_number: String::new(),
            game_mode: String::new(),
            game_type: String::new(),
            game_complexity: String::new(),
            tags: vec![],
            gravity: 0.5,
            play_area: 0.5,
            table: String::new(),
            sky: String::new(),
            note: String::new(),
            tab_states: HashMap::new(),
            lua_script: String::new(),
            lua_script_state: String::new(),
            xml_ui: String::new(),
            object_states,
        })
    }
}

/// Tabletop Simulator card types. See [the TTS API docs](https://api.tabletopsimulator.com/custom-game-objects/#custom-card).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardShape {
    RoundedRectangle,
    Rectangle,
    RoundedHexagon,
    Hexagon,
    Circle,
}

impl From<CardShape> for i64 {
    fn from(value: CardShape) -> Self {
        match value {
            CardShape::RoundedRectangle => 0,
            CardShape::Rectangle => 1,
            CardShape::RoundedHexagon => 2,
            CardShape::Hexagon => 3,
            CardShape::Circle => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(clippy::struct_excessive_bools)]
/// Partial implementation of Tabletop Simulator's Object State.  Many fields are missing, both because the [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated, and because this implementation is only meant to provide the minimum necessary for a deck of custom cards.
pub struct ObjectState {
    guid: String,
    name: String,
    transform: TransformState,
    nickname: String,
    description: String,
    #[serde(rename = "GMNotes")]
    gm_notes: String,
    alt_look_angle: Vector3,
    color_difuse: ColourState,
    layout_group_sort_index: i64,
    value: i64,
    locked: bool,
    grid: bool,
    snap: bool,
    #[serde(rename = "IgnoreFoW")]
    ignore_fow: bool,
    measure_movement: bool,
    drag_selectable: bool,
    autoraise: bool,
    sticky: bool,
    tooltip: bool,
    grid_projection: bool,
    hide_when_face_down: bool,
    hands: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    card_id: Option<i64>,
    sideways_card: bool,
    #[serde(rename = "DeckIDs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    deck_ids: Option<Vec<i64>>,
    custom_deck: HashMap<i64, CustomDeckState>,
    lua_script: String,
    lua_script_state: String,
    #[serde(rename = "XmlUI")]
    xml_ui: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    contained_objects: Option<Vec<ObjectState>>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
/// Partial implementation of Tabletop Simulator's `CustomDeckState`. The [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated, so fields may be missing.
pub struct CustomDeckState {
    #[serde(skip)]
    pub(super) name: String,
    pub(super) face_url: String,
    pub(super) back_url: String,
    pub(super) num_width: Option<i64>,
    pub(super) num_height: Option<i64>,
    pub(super) back_is_hidden: bool,
    pub(super) unique_back: bool,
    pub(super) r#type: i64,
}

type DeckData = (Vec<i64>, HashMap<i64, CustomDeckState>, Vec<ObjectState>);

fn generate_deck_data<T: GetCardInfo + Clone>(
    deck: Vec<CardEntry<T>>,
) -> Result<DeckData, CardError> {
    let mut card_ids = vec![];
    let mut custom_deck = HashMap::new();
    let mut contained_objects = vec![];
    let mut idx: i64 = 0;
    for card in deck {
        idx += 1;
        let id = idx * 100;
        custom_deck.insert(idx, card.get_custom_deck_state()?);
        for _ in 0..card.amount {
            card_ids.push(id);
            contained_objects.push(ObjectState {
                guid: generate_guid(),
                name: "CardCustom".to_string(),
                transform: TransformState::default(),
                nickname: String::new(),
                description: String::new(),
                gm_notes: String::new(),
                alt_look_angle: Vector3::default(),
                color_difuse: ColourState {
                    r: 0.713_235_259,
                    g: 0.713_235_259,
                    b: 0.713_235_259,
                },
                layout_group_sort_index: 0,
                value: 0,
                locked: false,
                grid: true,
                snap: true,
                ignore_fow: false,
                measure_movement: false,
                drag_selectable: true,
                autoraise: true,
                sticky: true,
                tooltip: true,
                grid_projection: false,
                hide_when_face_down: true,
                hands: true,
                card_id: Some(id),
                sideways_card: false,
                deck_ids: None,
                custom_deck: {
                    let mut hm = HashMap::new();
                    hm.insert(idx, card.get_custom_deck_state()?);
                    hm
                },
                lua_script: String::new(),
                lua_script_state: String::new(),
                xml_ui: String::new(),
                contained_objects: None,
            });
        }
    }
    Ok((card_ids, custom_deck, contained_objects))
}

/// Implementation of Tabletop Simulator's `TransformState`. While it would be strange for this structure to contain more fields than the ones in this implementation, fields may be missing because the [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct TransformState {
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub rot_x: f64,
    pub rot_y: f64,
    pub rot_z: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub scale_z: f64,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        }
    }
}

/// Implementation of Tabletop Simulator's Vector3. While it would be strange for this structure to contain more fields than the ones in this implementation, fields may be missing because the [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Implementation of Tabletop Simulator's `ColourState`. While it would be strange for this structure to contain more fields than the ones in this implementation, fields may be missing because the [knowledge base](https://kb.tabletopsimulator.com/custom-content/save-file-format/) is currently outdated.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct ColourState {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub enum SaveError {
    CouldntWriteObject { path: PathBuf, error: io::Error },
    CouldntWriteImage { path: PathBuf, error: io::Error },
    CouldntFindSaveDirectory,
}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CouldntWriteObject { path, error } => write!(
                f,
                "Failed to write the object at {} with error: {error}",
                path.display()
            ),
            Self::CouldntWriteImage { path, error } => write!(
                f,
                "Failed to write the image at {} with error: {error}",
                path.display()
            ),
            Self::CouldntFindSaveDirectory => {
                write!(f, "Couldn't find Tabletop Simulator's saved object files")
            }
        }
    }
}

/// Writes the object to the default TTS save directory. The image is mandatory.
/// # Errors
/// - If the object json file can't be written
/// - If the object image file can't be written
/// - If the default TTS save directory can't be found
pub fn write_to_tts_dir<P: AsRef<Path>, Cc: AsRef<[u8]>, Ci: AsRef<[u8]>>(
    output: P,
    contents: Cc,
    image: Ci,
) -> Result<(), SaveError> {
    let path = get_saved_objects_dir();
    match path {
        Some(mut path) => {
            path.push(output.as_ref());
            path.set_extension("json");
            match std::fs::write(path.clone(), contents) {
                Ok(()) => (),
                Err(error) => return Err(SaveError::CouldntWriteObject { path, error }),
            }
            path.set_extension("png");
            match std::fs::write(path.clone(), image) {
                Ok(()) => (),
                Err(error) => return Err(SaveError::CouldntWriteImage { path, error }),
            }
        }
        None => return Err(SaveError::CouldntFindSaveDirectory),
    }
    Ok(())
}

/// Gets the default saved objects directory for Tabletop Simulator. Implemented for Windows, Mac OS and Linux. The output value of this function is different depending on what OS it's been compiled for.
#[cfg(target_os = "windows")]
#[must_use]
pub fn get_saved_objects_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push("Documents\\My Games\\Tabletop Simulator\\Saves\\Saved Objects");
    }
    dir
}

/// Gets the default saved objects directory for Tabletop Simulator. Implemented for Windows, Mac OS and Linux. The output value of this function is different depending on what OS it's been compiled for.
#[cfg(target_os = "macos")]
#[must_use]
pub fn get_tts_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push("Library/Tabletop Simulator/Saves/Saved Objects");
    }
    dir
}

/// Gets the default saved objects directory for Tabletop Simulator. Implemented for Windows, Mac OS and Linux. The output value of this function is different depending on what OS it's been compiled for.
#[cfg(target_os = "linux")]
#[must_use]
pub fn get_tts_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push(".local/share/Tabletop Simulator/Saves/Saved Objects");
    }
    dir
}
