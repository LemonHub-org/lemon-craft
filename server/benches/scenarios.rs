use common::{
    ViewDistances,
    character::CharacterId,
    comp::{
        Collider, Density, Mass, Ori, Pos, Presence, PresenceKind, Vel,
        inventory::item::{MaterialStatManifest, tool::AbilityMap},
    },
    region::RegionMap,
    resources::GameMode,
    shared_server_config::ServerConstants,
    terrain::{MapSizeLg, TerrainChunk},
};
use common_ecs::run_now;
use common_state::State;
use common_systems::add_shared_systems;
use criterion::{Criterion, criterion_group, criterion_main};
use lemoncraft_server::{
    HwStats, Tick, TickStart,
    chunk_generator::ChunkGenerator,
    metrics::{
        ChunkGenMetrics, EcsSystemMetrics, JobMetrics, PhysicsMetrics, QueryServerMetrics,
        TickMetrics,
    },
    sys::metrics::Sys as MetricsSys,
};
use prometheus::Registry;
use rand::RngExt;
use specs::{Builder, Join, WorldExt};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use vek::{Vec2, Vec3};

const DT: Duration = Duration::from_millis(20);
const SERVER_CONSTANTS: ServerConstants = ServerConstants {
    day_cycle_coefficient: 24.0,
};

/// Build a minimal server ECS: common systems + the server-side resources
/// (region map, presence, etc.) that the tick loop depends on.
fn setup() -> State {
    let pools = State::pools(GameMode::Server);
    let map_size_lg = MapSizeLg::new(Vec2::new(10, 10)).unwrap();
    let mut state = State::server(
        pools,
        map_size_lg,
        Arc::new(TerrainChunk::water(0)),
        add_shared_systems,
    );
    // Resources needed by the systems we registered above.
    state.ecs_mut().insert(MaterialStatManifest::with_empty());
    state.ecs_mut().insert(AbilityMap::with_empty());
    // Server-side resources (normally inserted by `Server::new`).
    state.ecs_mut().register::<Presence>();
    state.ecs_mut().insert(RegionMap::new());
    state
}

/// Resources required by `sys::metrics::Sys`.
fn setup_metrics_resources(state: &mut State) {
    let registry = Registry::new();
    state.ecs_mut().insert(HwStats::new(4, 8));
    state.ecs_mut().insert(Tick::new(0));
    state.ecs_mut().insert(TickStart::new(Instant::now()));
    state.ecs_mut().insert(ChunkGenerator::new(
        ChunkGenMetrics::new(&registry).unwrap(),
    ));
    state
        .ecs_mut()
        .insert(EcsSystemMetrics::new(&registry).unwrap());
    state.ecs_mut().insert(TickMetrics::new(&registry).unwrap());
    state
        .ecs_mut()
        .insert(PhysicsMetrics::new(&registry).unwrap());
    state.ecs_mut().insert(JobMetrics::new(&registry).unwrap());
    state
        .ecs_mut()
        .insert(QueryServerMetrics::new(&registry).unwrap());
}

fn add_entities(state: &mut State, count: u32, moving: bool) {
    let mut rng = rand::rng();
    for i in 0..count {
        let vel = if moving {
            Vel(Vec3::new(
                rng.random_range(-5.0..5.0),
                rng.random_range(-5.0..5.0),
                0.0,
            ))
        } else {
            Vel::zero()
        };
        state
            .ecs_mut()
            .create_entity()
            .with(Pos(Vec3::new(i as f32 * 3.0, 0.0, 100.0)))
            .with(vel)
            .with(Ori::default())
            .with(Mass(1.0))
            .with(Density(1.0))
            .with(Collider::Point)
            .with(Presence::new(
                ViewDistances {
                    terrain: 16,
                    entity: 16,
                },
                PresenceKind::Character(CharacterId(i as i64)),
            ))
            .build();
    }
}

/// Advance moving entities by `vel * dt`, simulating movement that would
/// otherwise be produced by the physics system.
fn advance_entities(state: &mut State) {
    let ecs = state.ecs_mut();
    let mut positions = ecs.write_storage::<Pos>();
    let velocities = ecs.read_storage::<Vel>();
    for (pos, vel) in (&mut positions, &velocities).join() {
        pos.0 += vel.0 * DT.as_secs_f32();
    }
}

/// A full server tick as done by `Server::tick` minus the parts that require a
/// network connection or world generation.
fn server_tick(state: &mut State, moving: bool, with_metrics: bool) {
    if moving {
        advance_entities(state);
    }
    state.tick(DT, false, None, &SERVER_CONSTANTS, |_, _| {});
    // Mimic `Server::tick` bookkeeping.
    let ecs = state.ecs_mut();
    *ecs.write_resource::<Tick>() = Tick::new(ecs.read_resource::<Tick>().get() + 1);
    *ecs.write_resource::<TickStart>() = TickStart::new(Instant::now());
    ecs.write_resource::<RegionMap>().tick(
        ecs.read_storage::<Pos>(),
        ecs.read_storage::<Vel>(),
        ecs.read_storage::<Presence>(),
        ecs.entities(),
    );
    if with_metrics {
        run_now::<MetricsSys>(ecs);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("server/empty");
        let mut state = setup();
        group.bench_function("tick", |b| {
            b.iter(|| server_tick(&mut state, false, false));
        });
        let mut state = setup();
        setup_metrics_resources(&mut state);
        group.bench_function("tick_with_metrics", |b| {
            b.iter(|| server_tick(&mut state, false, true));
        });
        group.finish();
    }

    for count in [1_000u32, 5_000, 10_000] {
        let mut group = c.benchmark_group(format!("server/entities_{count}"));

        let mut state = setup();
        add_entities(&mut state, count, false);
        group.bench_function("static", |b| {
            b.iter(|| server_tick(&mut state, false, false));
        });

        let mut state = setup();
        add_entities(&mut state, count, true);
        group.bench_function("moving", |b| {
            b.iter(|| server_tick(&mut state, true, false));
        });

        group.finish();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
