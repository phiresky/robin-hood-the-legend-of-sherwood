//! Entity identity. A tiny module on its own because virtually every
//! sim module references `EntityId` and pulling in all of `element` to
//! get one `u32` newtype is not justified.

use serde::{Deserialize, Serialize};

/// Unique identifier for an entity in the game world.
///
/// Stored as a 0-based index into the engine's entity table. Debug/Display
/// formatting prints both the internal 0-based index and the 1-based
/// script-handle number exposed by `GetActorScript(N)`
/// (handle = `EntityId(n).0 + 1`), so log output carries both identifiers.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    robin_state_hash_derive::StateHash,
)]
pub struct EntityId(pub u32);

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({}/script={})", self.0, self.0 + 1)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({}/script={})", self.0, self.0 + 1)
    }
}
