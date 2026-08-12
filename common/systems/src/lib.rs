#![expect(clippy::option_map_unit_fn)]

mod arcing;
mod aura;
mod beam;
mod buff;
pub mod character_behavior;
pub mod controller;
mod interpolation;
pub mod melee;
mod mount;
pub mod phys;
mod phys_events;
mod pool;
pub mod projectile;
mod shockwave;
mod stats;
mod tether;

// External
use common_ecs::{System, dispatch};
use specs::DispatcherBuilder;

pub fn add_local_systems(dispatch_builder: &mut DispatcherBuilder) {
    add_client_systems(dispatch_builder);
}

/// Registers the full client-side system set, including the entity
/// interpolation system (physics depends on it there).
pub fn add_client_systems(dispatch_builder: &mut DispatcherBuilder) {
    dispatch::<interpolation::Sys>(dispatch_builder, &[]);
    add_common_systems(dispatch_builder, true);
}

/// Registers systems shared between client and server. Unlike
/// [`add_client_systems`], this does not register the interpolation system
/// (which is a client-only concern) and physics does not depend on it.
pub fn add_shared_systems(dispatch_builder: &mut DispatcherBuilder) {
    add_common_systems(dispatch_builder, false);
}

fn add_common_systems(dispatch_builder: &mut DispatcherBuilder, with_interpolation: bool) {
    dispatch::<tether::Sys>(dispatch_builder, &[]);
    dispatch::<mount::Sys>(dispatch_builder, &[]);
    dispatch::<controller::Sys>(dispatch_builder, &[&mount::Sys::sys_name()]);
    dispatch::<character_behavior::Sys>(dispatch_builder, &[&controller::Sys::sys_name()]);
    dispatch::<buff::Sys>(dispatch_builder, &[]);
    dispatch::<stats::Sys>(dispatch_builder, &[&buff::Sys::sys_name()]);
    if with_interpolation {
        dispatch::<phys::Sys>(dispatch_builder, &[
            &interpolation::Sys::sys_name(),
            &controller::Sys::sys_name(),
            &mount::Sys::sys_name(),
            &stats::Sys::sys_name(),
        ]);
    } else {
        dispatch::<phys::Sys>(dispatch_builder, &[
            &controller::Sys::sys_name(),
            &mount::Sys::sys_name(),
            &stats::Sys::sys_name(),
        ]);
    }
    dispatch::<phys_events::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<projectile::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<shockwave::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<arcing::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<beam::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<pool::Sys>(dispatch_builder, &[&phys::Sys::sys_name()]);
    dispatch::<aura::Sys>(dispatch_builder, &[]);
}
