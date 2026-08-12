use crate::{
    HwStats, Tick, TickStart,
    chunk_generator::ChunkGenerator,
    metrics::{EcsSystemMetrics, JobMetrics, PhysicsMetrics, QueryServerMetrics, TickMetrics},
};
use common::{resources::TimeOfDay, slowjob::SlowJobPool, terrain::TerrainGrid};
use common_ecs::{Job, Origin, Phase, SysMetrics, System};
use lemoncraft_query_server::server::Metrics as RawQueryServerMetrics;
use specs::{Entities, Join, Read, ReadExpect};
use std::{
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

/// How often (in ticks) the per-system ECS metrics are aggregated and
/// exported to Prometheus. Aggregation (`gen_stats`) is comparatively
/// expensive: it combines the timelines of all systems for every measurement
/// point, so it only needs to run often enough to keep slow systems visible.
/// Overridable via the `VELOREN_METRICS_SAMPLE_INTERVAL` environment variable.
const METRIC_SAMPLE_INTERVAL_TICKS_DEFAULT: u64 = 20;
/// A tick that takes longer than this is considered slow and is always
/// sampled, even between regular sampling intervals.
const SLOW_TICK_THRESHOLD: Duration = Duration::from_millis(50);

fn metric_sample_interval_ticks() -> u64 {
    static INTERVAL: OnceLock<u64> = OnceLock::new();
    *INTERVAL.get_or_init(|| {
        std::env::var("VELOREN_METRICS_SAMPLE_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|v| *v > 0)
            .unwrap_or(METRIC_SAMPLE_INTERVAL_TICKS_DEFAULT)
    })
}

/// This system exports metrics
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, HwStats>,
        ReadExpect<'a, Tick>,
        ReadExpect<'a, TimeOfDay>,
        ReadExpect<'a, TickStart>,
        ReadExpect<'a, ChunkGenerator>,
        Option<Read<'a, TerrainGrid>>,
        Read<'a, SysMetrics>,
        Read<'a, common_ecs::PhysicsMetrics>,
        ReadExpect<'a, SlowJobPool>,
        ReadExpect<'a, EcsSystemMetrics>,
        ReadExpect<'a, TickMetrics>,
        ReadExpect<'a, PhysicsMetrics>,
        ReadExpect<'a, JobMetrics>,
        Option<Read<'a, Arc<Mutex<RawQueryServerMetrics>>>>,
        ReadExpect<'a, QueryServerMetrics>,
    );

    const NAME: &'static str = "metrics";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Apply;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            hw_stats,
            tick,
            time_of_day,
            tick_start,
            chunk_generator,
            terrain,
            sys_metrics,
            phys_metrics,
            slowjobpool,
            export_ecs,
            export_tick,
            export_physics,
            export_jobs,
            raw_query_server,
            export_query_server,
        ): Self::SystemData,
    ) {
        const NANOSEC_PER_SEC: f64 = std::time::Duration::from_secs(1).as_nanos() as f64;

        let start = Instant::now();

        // Aggregate the per-system timelines only every few ticks: `gen_stats`
        // iterates all measurement points of all systems, which is
        // comparatively expensive to do every tick. Slow ticks are always
        // sampled so that spikes remain visible.
        let sample_ecs_metrics = tick.0 % metric_sample_interval_ticks() == 0
            || tick_start.0.elapsed() > SLOW_TICK_THRESHOLD;

        // Only needed for the tracy-gated plots below.
        #[cfg(not(feature = "tracy"))]
        let _ = chunk_generator;

        let mut state = sys_metrics.stats.lock().unwrap();
        //this system hasn't run yet
        state.remove(Self::NAME);

        if sample_ecs_metrics {
            for (name, stat) in common_ecs::gen_stats(
                &state,
                tick_start.0,
                hw_stats.rayon_threads,
                hw_stats.hardware_threads,
            ) {
                export_ecs
                    .system_start_time
                    .with_label_values(&[name])
                    .set(stat.start_ns() as i64);
                export_ecs
                    .system_thread_avg
                    .with_label_values(&[name])
                    .set(stat.avg_threads() as f64);
                let len = stat.length_ns();
                export_ecs
                    .system_length_time
                    .with_label_values(&[name])
                    .set(len as i64);
                export_ecs
                    .system_length_count
                    .with_label_values(&[name])
                    .inc_by(len);
                export_ecs
                    .system_length_hist
                    .with_label_values(&[name])
                    .observe(len as f64 / NANOSEC_PER_SEC);
            }
        }

        // Report other info
        export_tick.time_of_day.set(time_of_day.0);
        if tick.0.rem_euclid(100) == 0 {
            if let Some(terrain) = terrain.as_ref() {
                let mut chonk_cnt = 0;
                let mut group_cnt = 0;
                let chunk_cnt = terrain.iter().fold(0, |a, (_, c)| {
                    chonk_cnt += 1;
                    group_cnt += c.sub_chunk_groups();
                    a + c.sub_chunks_len()
                });
                export_tick.chonks_count.set(chonk_cnt as i64);
                export_tick.chunks_count.set(chunk_cnt as i64);
                export_tick.chunk_groups_count.set(group_cnt as i64);
            }

            let entity_count = entities.join().count();
            export_tick.entity_count.set(entity_count as i64);
        }
        // These plots are only meaningful in profiling builds; in normal builds
        // the values would be expensive full scans (entities, pending chunks,
        // terrain) executed every tick for no observer. The equivalent
        // counts are exported to Prometheus every 100 ticks above.
        #[cfg(feature = "tracy")]
        {
            common_base::plot!("entity count", entities.join().count() as f64);
            common_base::plot!(
                "pending chunks",
                chunk_generator.pending_chunks().count() as f64
            );
            if let Some(terrain) = terrain.as_ref() {
                common_base::plot!("chunk count", terrain.iter().count() as f64);
            }
        }

        //detailed physics metrics
        export_physics
            .entity_entity_collision_checks_count
            .inc_by(phys_metrics.entity_entity_collision_checks);
        export_physics
            .entity_entity_collisions_count
            .inc_by(phys_metrics.entity_entity_collisions);

        //detailed job metrics
        for (name, jobs) in slowjobpool.take_metrics() {
            let queried = export_jobs.job_queried_hst.with_label_values(&[&name]);
            let executed = export_jobs.job_execution_hst.with_label_values(&[&name]);
            for job in jobs {
                queried.observe(
                    job.execution_start
                        .duration_since(job.queue_created)
                        .as_secs_f64(),
                );
                executed.observe(
                    job.execution_end
                        .duration_since(job.execution_start)
                        .as_secs_f64(),
                );
            }
        }

        // export self time as best as possible
        export_ecs
            .system_start_time
            .with_label_values(&["metrics"])
            .set(start.duration_since(tick_start.0).as_nanos() as i64);
        export_ecs
            .system_thread_avg
            .with_label_values(&["metrics"])
            .set(1.0);
        let len = start.elapsed().as_nanos() as u64;
        export_ecs
            .system_length_time
            .with_label_values(&["metrics"])
            .set(len as i64);
        export_ecs
            .system_length_count
            .with_label_values(&["metrics"])
            .inc_by(len);
        export_ecs
            .system_length_hist
            .with_label_values(&["metrics"])
            .observe(len as f64 / NANOSEC_PER_SEC);

        if let Some(Ok(metrics)) = raw_query_server
            .as_ref()
            // Hold the lock for the shortest time possible
            .map(|m| m.lock().map(|mut metrics| metrics.reset()))
        {
            export_query_server.apply(metrics);
        }
    }
}
