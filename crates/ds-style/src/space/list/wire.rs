//! `spaces.json` as the model reads and writes it.
//!
//! Reading is lenient where mailo's was: a Space with a missing or unknown field takes that
//! field's default, dots are held to one to three with finite values, a missing grain is the
//! preset's own for the Space's place, `current` is held to a real Space, and a damaged
//! `recall` map costs only itself. A file with no Space at all is not a `Spaces` (the caller
//! falls back to a first run). A Space with no `id`, as mailo wrote before the kit, takes its
//! position, so keys that were positions (`recall`, Today's `space`) keep meaning the same Space.

use super::model::{Space, SpaceId, Spaces};
use super::recall::Recall;
use crate::appearance::theme::Theme;
use crate::space::look::{CardAccent, Grain, SpaceLook};
use crate::space::palette::{Dot, NEUTRAL_DOT};
use crate::space::presets::default_look;
use ds_core::word::Word;
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The most dots a Space holds. The gradient reads as a gradient up to three.
pub const MOST_DOTS: usize = 3;

/// `look` held to the limits: one to three dots, each with a finite hue and chroma in range.
pub(super) fn clamp_look(mut look: SpaceLook) -> SpaceLook {
    look.dots = look
        .dots
        .into_iter()
        .map(clamp_dot)
        .take(MOST_DOTS)
        .collect();
    if look.dots.is_empty() {
        look.dots.push(NEUTRAL_DOT);
    }
    look
}

fn clamp_dot(dot: Dot) -> Dot {
    Dot {
        hue: if dot.hue.is_finite() {
            dot.hue.rem_euclid(360.0)
        } else {
            NEUTRAL_DOT.hue
        },
        chroma: if dot.chroma.is_finite() {
            dot.chroma.clamp(0.0, 1.0)
        } else {
            NEUTRAL_DOT.chroma
        },
    }
}

#[derive(Serialize)]
struct SpaceOut<'a, P> {
    id: SpaceId,
    name: &'a str,
    #[serde(flatten)]
    look: &'a SpaceLook,
    #[serde(flatten)]
    payload: &'a P,
}

#[derive(Serialize)]
struct SpacesOut<'a, P, R> {
    spaces: Vec<SpaceOut<'a, P>>,
    /// The position of the Space on screen.
    current: usize,
    recall: &'a BTreeMap<SpaceId, R>,
}

impl<P: Serialize, R: Serialize> Serialize for Spaces<P, R> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SpacesOut {
            spaces: self
                .list
                .iter()
                .map(|space| SpaceOut {
                    id: space.id,
                    name: &space.name,
                    look: &space.look,
                    payload: &space.payload,
                })
                .collect(),
            current: self.index_of(self.current).unwrap_or(0),
            recall: &self.recall.0,
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
struct SpaceRaw<P> {
    #[serde(default, deserialize_with = "lenient_id")]
    id: Option<SpaceId>,
    #[serde(default)]
    name: String,
    #[serde(default, deserialize_with = "lenient_dots")]
    dots: Vec<Dot>,
    #[serde(default, deserialize_with = "lenient_grain")]
    grain: Option<u8>,
    #[serde(default, deserialize_with = "lenient_theme")]
    theme: Theme,
    #[serde(default, deserialize_with = "lenient_accent")]
    card_accent: CardAccent,
    #[serde(flatten)]
    payload: P,
}

#[derive(Deserialize)]
struct SpacesRaw<P> {
    #[serde(default = "Vec::new")]
    spaces: Vec<SpaceRaw<P>>,
    #[serde(default, deserialize_with = "lenient_index")]
    current: usize,
    #[serde(default)]
    recall: Value,
}

impl<'de, P: DeserializeOwned, R: DeserializeOwned> Deserialize<'de> for Spaces<P, R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = SpacesRaw::<P>::deserialize(deserializer)?;
        if raw.spaces.is_empty() {
            return Err(D::Error::custom("no Spaces"));
        }
        let ids = assigned_ids(raw.spaces.iter().map(|space| space.id));
        let list: Vec<Space<P>> = raw
            .spaces
            .into_iter()
            .zip(ids)
            .enumerate()
            .map(|(index, (space, id))| from_raw(space, id, index))
            .collect();
        let current = list[raw.current.min(list.len() - 1)].id;
        let known: BTreeSet<SpaceId> = list.iter().map(|space| space.id).collect();
        let recall = serde_json::from_value::<BTreeMap<SpaceId, R>>(raw.recall)
            .unwrap_or_default()
            .into_iter()
            .filter(|(id, _)| known.contains(id))
            .collect();
        let next = list.iter().map(|space| space.id.0 + 1).max().unwrap_or(0);
        Ok(Spaces {
            list,
            current,
            recall: Recall(recall),
            next,
        })
    }
}

fn from_raw<P>(raw: SpaceRaw<P>, id: SpaceId, index: usize) -> Space<P> {
    let grain = raw.grain.map_or_else(
        || default_look(index, Grain::default(), CardAccent::default()).grain,
        Grain,
    );
    Space {
        id,
        name: raw.name,
        look: clamp_look(SpaceLook {
            dots: raw.dots,
            grain,
            theme: raw.theme,
            card_accent: raw.card_accent,
        }),
        payload: raw.payload,
    }
}

/// One id per Space: the stored one when it is the first to use it, else the Space's position,
/// else the next number nobody has.
fn assigned_ids(stored: impl Iterator<Item = Option<SpaceId>>) -> Vec<SpaceId> {
    let stored: Vec<Option<SpaceId>> = stored.collect();
    let mut taken: BTreeSet<u64> = BTreeSet::new();
    let mut kept: Vec<Option<SpaceId>> = Vec::new();
    for id in &stored {
        kept.push(id.filter(|id| taken.insert(id.0)));
    }
    let mut spare = taken.iter().next_back().map_or(0, |top| top + 1);
    kept.into_iter()
        .enumerate()
        .map(|(index, id)| {
            id.unwrap_or_else(|| {
                let by_position = index as u64;
                let free = if taken.contains(&by_position) {
                    spare += 1;
                    spare - 1
                } else {
                    by_position
                };
                taken.insert(free);
                SpaceId(free)
            })
        })
        .collect()
}

fn lenient_id<'de, D: Deserializer<'de>>(d: D) -> Result<Option<SpaceId>, D::Error> {
    Ok(Value::deserialize(d)?.as_u64().map(SpaceId))
}

fn lenient_dots<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Dot>, D::Error> {
    Ok(serde_json::from_value(Value::deserialize(d)?).unwrap_or_default())
}

fn lenient_index<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    let value = Value::deserialize(d)?;
    Ok(value
        .as_u64()
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(0))
}

fn lenient_grain<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u8>, D::Error> {
    Ok(Value::deserialize(d)?
        .as_u64()
        .map(|grain| u8::try_from(grain).unwrap_or(u8::MAX).min(100)))
}

fn lenient_theme<'de, D: Deserializer<'de>>(d: D) -> Result<Theme, D::Error> {
    Ok(Value::deserialize(d)?
        .as_str()
        .and_then(Theme::parse)
        .unwrap_or_default())
}

/// `hint` is what mailo wrote before the look was quire's and `space_hue` is quire's word for
/// the same choice; every other word is the chosen accent.
fn lenient_accent<'de, D: Deserializer<'de>>(d: D) -> Result<CardAccent, D::Error> {
    Ok(match Value::deserialize(d)?.as_str() {
        Some("hint" | "space_hue") => CardAccent::SpaceHue,
        _ => CardAccent::Chosen,
    })
}
