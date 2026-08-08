//! This crate contains the [`State`] and shared between
//! server (`lemoncraft-server`) and the client (`lemoncraft-client`)

mod special_areas;
mod state;
// TODO: breakup state module and remove glob
pub use special_areas::*;
pub use state::{BlockChange, BlockDiff, ScheduledBlockChange, State, TerrainChanges};
