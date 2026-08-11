use crate::uid::Uid;
use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage};

/// Marks an item drop with the Uid of the NPC that dropped it. Server-side
/// only; used to let a living NPC verbally warn a player who takes its loot
/// (no aggression — see the tolerance design).
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ItemSource(pub Option<Uid>);

impl Component for ItemSource {
    type Storage = DerefFlaggedStorage<Self, specs::DenseVecStorage<Self>>;
}
