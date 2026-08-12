use common::terrain::BlockKind;
use hashbrown::HashMap;
use vek::*;

/// Server-side break progress for terrain blocks being mined.
///
/// Filled terrain blocks cannot store the `Damage` sprite attribute (sprite
/// attributes only exist on non-filled blocks), so mining progress is tracked
/// here, keyed by world position. Progress is shared between all players and
/// decays after [`BlockDamage::DECAY_SECS`] without any hit, mirroring the
/// Minecraft crack-decay behaviour. The map is bounded: entries are removed
/// on break, and expired entries are pruned once the map grows large.
#[derive(Default)]
pub struct BlockDamage {
    entries: HashMap<Vec3<i32>, Entry>,
}

/// How long (in real seconds) without a hit before break progress resets.
pub const BLOCK_DAMAGE_DECAY_SECS: f64 = 5.0;

/// Prune the map when it holds more than this many entries.
const PRUNE_THRESHOLD: usize = 512;

/// Damage required to break each terrain-breakable block kind.
/// Non-breakable kinds return 0 (never break).
#[inline]
pub const fn required_damage(kind: BlockKind) -> u8 {
    match kind {
        BlockKind::Rock => 4,
        BlockKind::WeakRock => 2,
        BlockKind::Grass => 2,
        BlockKind::Snow => 1,
        BlockKind::Earth => 2,
        BlockKind::Sand => 1,
        BlockKind::Wood => 3,
        BlockKind::Leaves => 1,
        BlockKind::Ice => 2,
        _ => 0,
    }
}

/// Number of crack stages shown while mining (MC-style).
pub const CRACK_STAGES: u8 = 4;

/// Crack stage (0..=4) for the given damage, matching `CRACK_STAGES`.
#[inline]
pub fn crack_stage(damage: u8, required: u8) -> u8 {
    let required = required.max(1);
    (damage * CRACK_STAGES).div_ceil(required).min(CRACK_STAGES)
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    damage: u8,
    last_hit: f64,
}

/// Result of a single mining hit on a terrain block.
#[derive(Clone, Copy, Debug)]
pub struct BreakProgress {
    pub damage: u8,
    /// Whether the crack stage advanced with this hit.
    pub stage_changed: bool,
    /// Whether the block broke and the entry was removed.
    pub broken: bool,
    /// Current crack stage (0..=4).
    pub stage: u8,
}

impl BlockDamage {
    /// Register a hit on the block at `pos`, returning the new progress.
    pub fn hit(&mut self, pos: Vec3<i32>, time: f64, kind: BlockKind) -> Option<BreakProgress> {
        let required = required_damage(kind);
        if required == 0 {
            return None;
        }

        let entry = self.entries.entry(pos).or_insert(Entry {
            damage: 0,
            last_hit: time,
        });
        // Decay progress that has sat idle for too long.
        if time - entry.last_hit > BLOCK_DAMAGE_DECAY_SECS {
            entry.damage = 0;
        }
        entry.last_hit = time;
        entry.damage += 1;

        let old_stage = crack_stage(entry.damage.saturating_sub(1), required);
        let new_stage = crack_stage(entry.damage, required);

        if entry.damage >= required {
            let progress = BreakProgress {
                damage: entry.damage,
                stage_changed: true,
                broken: true,
                stage: new_stage,
            };
            self.entries.remove(&pos);
            Some(progress)
        } else {
            Some(BreakProgress {
                damage: entry.damage,
                stage_changed: new_stage > old_stage,
                broken: false,
                stage: new_stage,
            })
        }
    }

    /// Remove expired entries once the map grows past the prune threshold.
    /// Keeps the registry bounded when players abandon partially mined blocks.
    pub fn prune_expired(&mut self, time: f64) {
        if self.entries.len() >= PRUNE_THRESHOLD {
            self.entries
                .retain(|_, e| time - e.last_hit <= BLOCK_DAMAGE_DECAY_SECS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_damage() {
        assert_eq!(required_damage(BlockKind::Sand), 1);
        assert_eq!(required_damage(BlockKind::Rock), 4);
        assert_eq!(required_damage(BlockKind::Wood), 3);
        assert_eq!(required_damage(BlockKind::Air), 0);
        assert_eq!(required_damage(BlockKind::Water), 0);
    }

    #[test]
    fn test_crack_stages() {
        // 4 stages spread across the hardness.
        assert_eq!(crack_stage(0, 4), 0);
        assert_eq!(crack_stage(1, 4), 1);
        assert_eq!(crack_stage(2, 4), 2);
        assert_eq!(crack_stage(3, 4), 3);
        assert_eq!(crack_stage(4, 4), 4);
        // One-hit blocks jump straight to the final stage.
        assert_eq!(crack_stage(1, 1), 4);
        // Never exceed the max stage.
        assert_eq!(crack_stage(9, 4), 4);
    }

    #[test]
    fn test_hit_accumulates_and_breaks() {
        let mut damage = BlockDamage::default();
        let pos = Vec3::new(1, 2, 3);

        let first = damage.hit(pos, 0.0, BlockKind::Rock).unwrap();
        assert!(!first.broken);
        assert_eq!(first.damage, 1);
        assert_eq!(first.stage, 1);

        let mid = damage.hit(pos, 0.1, BlockKind::Rock).unwrap();
        assert!(!mid.broken);
        assert_eq!(mid.damage, 2);
        assert_eq!(mid.stage, 2);

        let third = damage.hit(pos, 0.2, BlockKind::Rock).unwrap();
        assert!(!third.broken);
        assert_eq!(third.damage, 3);
        assert_eq!(third.stage, 3);

        let last = damage.hit(pos, 0.3, BlockKind::Rock).unwrap();
        assert!(last.broken);
        assert_eq!(last.damage, 4);
        // Entry is removed on break.
        assert_eq!(damage.hit(pos, 0.4, BlockKind::Rock).unwrap().damage, 1);
    }

    #[test]
    fn test_decay_resets_progress() {
        let mut damage = BlockDamage::default();
        let pos = Vec3::new(1, 2, 3);

        damage.hit(pos, 0.0, BlockKind::Rock);
        let after_decay = damage
            .hit(pos, BLOCK_DAMAGE_DECAY_SECS + 1.0, BlockKind::Rock)
            .unwrap();
        assert_eq!(after_decay.damage, 1, "idle progress must reset");
    }

    #[test]
    fn test_non_breakable_ignored() {
        let mut damage = BlockDamage::default();
        assert!(
            damage
                .hit(Vec3::new(0, 0, 0), 0.0, BlockKind::Lava)
                .is_none()
        );
    }
}
