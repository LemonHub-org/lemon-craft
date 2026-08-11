use common::{
    comp::{Health, Inventory, PickupItem, Player, Pos},
    event::{EventBus, InventoryManipEvent},
    resources::ProgramTime,
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use specs::{Entities, Entity, Join, Read, ReadExpect, ReadStorage};
use std::{collections::HashMap, sync::Mutex};

/// Distance at which a player automatically picks up items on the ground.
const AUTO_PICKUP_RANGE: f32 = 2.5;
/// Items younger than this are left alone (so freshly dropped loot is not
/// instantly vacuumed back up).
const ITEM_MIN_AGE_SECS: f64 = 1.5;
/// Minimum time between automatic pickup attempts per player, to avoid
/// spamming pickup events when the inventory is full.
const PICKUP_ATTEMPT_COOLDOWN: f64 = 0.4;

static LAST_ATTEMPT: Mutex<Option<HashMap<Entity, f64>>> = Mutex::new(None);

/// Automatically picks up nearby item drops for living players.
#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, PickupItem>,
        ReadStorage<'a, Uid>,
        Read<'a, EventBus<InventoryManipEvent>>,
        ReadExpect<'a, ProgramTime>,
    );

    const NAME: &'static str = "auto_pickup";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Apply;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            positions,
            healths,
            inventories,
            players,
            items,
            uids,
            event_bus,
            program_time,
        ): Self::SystemData,
    ) {
        let now = program_time.0;
        let mut emitter = event_bus.emitter();

        let mut attempts = LAST_ATTEMPT.lock().expect("auto_pickup state poisoned");

        for (player_entity, player_pos, health, _inventory, _player) in
            (&entities, &positions, &healths, &inventories, &players).join()
        {
            // Skip dead players.
            if health.fraction() <= 0.0 {
                continue;
            }

            // Cooldown per player to avoid event spam when the inventory is
            // full or nothing is pickable.
            let last = attempts
                .as_ref()
                .and_then(|map| map.get(&player_entity))
                .copied();
            if last.is_some_and(|t| now - t < PICKUP_ATTEMPT_COOLDOWN) {
                continue;
            }

            let mut picked_up = false;
            for (_item_entity, item_pos, item, uid) in (&entities, &positions, &items, &uids).join()
            {
                // Only pick up items old enough to be fair game.
                if now - item.created().0 < ITEM_MIN_AGE_SECS {
                    continue;
                }
                let dist_sqrd = item_pos.0.distance_squared(player_pos.0);
                if dist_sqrd > AUTO_PICKUP_RANGE * AUTO_PICKUP_RANGE {
                    continue;
                }
                emitter.emit(InventoryManipEvent(
                    player_entity,
                    common::comp::InventoryManip::Pickup(*uid),
                ));
                picked_up = true;
            }

            if picked_up {
                let map = attempts.get_or_insert_with(HashMap::new);
                map.insert(player_entity, now);
            }
        }
    }
}
