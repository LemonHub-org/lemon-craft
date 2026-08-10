use crate::{comp::inventory::item::ItemDefinitionIdOwned, uid::Uid};
use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage};

/// Marks an item drop with the Uid of the NPC that dropped it. Server-side
/// only; used to trigger loot-grudge retaliation when a player picks it up.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ItemSource(pub Option<Uid>);

impl Component for ItemSource {
    type Storage = DerefFlaggedStorage<Self, specs::DenseVecStorage<Self>>;
}

/// One stolen item: which player took it, and what item was taken.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootGrudgeEntry {
    pub thief: Uid,
    pub item: ItemDefinitionIdOwned,
}

/// A grudge held by an NPC against players who picked up its loot. The NPC
/// attacks those players until they return the stolen items (drop them near
/// the NPC) or the players die.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LootGrudge(pub Vec<LootGrudgeEntry>);

impl Component for LootGrudge {
    type Storage = DerefFlaggedStorage<Self, specs::VecStorage<Self>>;
}
