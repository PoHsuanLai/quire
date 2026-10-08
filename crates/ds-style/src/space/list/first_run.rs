//! The Spaces a first run starts with.

use super::model::{Space, SpaceId, Spaces};
use crate::space::look::{CardAccent, Grain};
use crate::space::presets::{PRESETS, default_look};

impl<P, R> Spaces<P, R> {
    /// One Space per payload, named "Space 1" onward and tinted from the presets in order.
    /// Given none, one Space over `fallback`, so the window always has somewhere to be.
    pub fn first_run(payloads: impl IntoIterator<Item = P>, fallback: impl FnOnce() -> P) -> Self {
        let mut made = payloads
            .into_iter()
            .enumerate()
            .map(|(index, payload)| Space {
                id: SpaceId(index as u64),
                name: format!("Space {}", index + 1),
                link: None,
                look: default_look(index, Grain::default(), CardAccent::default()),
                payload,
            });
        let first = made.next().unwrap_or_else(|| Space {
            id: SpaceId(0),
            name: "Space 1".to_owned(),
            link: None,
            look: default_look(0, Grain::default(), CardAccent::default()),
            payload: fallback(),
        });
        let mut spaces = Spaces::new(first);
        for space in made {
            spaces.next = space.id.0 + 1;
            spaces.list.push(space);
        }
        spaces
    }

    /// The preset names, in order: what the Space editor calls each.
    pub fn preset_names() -> [&'static str; 8] {
        std::array::from_fn(|index| PRESETS[index].name)
    }
}
