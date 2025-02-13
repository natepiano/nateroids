use crate::{
    camera::stars::{Star, StarConfig},
    schedule::InGameSet,
};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

pub struct StarTwinklingPlugin;

impl Plugin for StarTwinklingPlugin {
    fn build(&self, app: &mut App) {
        let start_twinkling_timer_duration = StarConfig::default().start_twinkling_delay;

        app.init_resource::<StarConfig>()
            .insert_resource(StartTwinklingTimer {
                timer: Timer::from_seconds(start_twinkling_timer_duration, TimerMode::Repeating),
            })
            .add_systems(
                Update,
                ((start_twinkling, update_twinkling),).in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Component)]
struct Twinkling {
    original_emissive: Vec4,
    target_emissive: Vec4,
    twinkle_timer: Timer,
}

#[derive(Resource)]
struct StartTwinklingTimer {
    timer: Timer,
}

fn should_start_twinkling(start_timer: &mut ResMut<StartTwinklingTimer>, time: Res<Time>) -> bool {
    start_timer.timer.tick(time.delta());
    if !start_timer.timer.just_finished() {
        return false;
    }
    true
}

fn get_random_indices(count: usize, range: usize) -> Vec<usize> {
    let mut rng = rand::rng();
    let mut numbers = HashSet::with_capacity(count);
    while numbers.len() < count {
        numbers.insert(rng.random_range(0..range));
    }
    numbers.into_iter().collect()
}

fn extract_elements_at_indices<T: Clone>(vec: &[T], indices: &[usize]) -> Vec<T> {
    indices.iter().filter_map(|&i| vec.get(i).cloned()).collect()
}

// yeah - but how can the query below be much simpler?
#[allow(clippy::type_complexity)]
fn start_twinkling(
    mut commands: Commands,
    config: Res<StarConfig>,
    stars: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (With<Star>, Without<Twinkling>)>,
    materials: Res<Assets<StandardMaterial>>,
    mut start_timer: ResMut<StartTwinklingTimer>,
    time: Res<Time>,
) {
    if !should_start_twinkling(&mut start_timer, time) {
        return;
    }

    let indices = get_random_indices(config.twinkle_choose_multiple_count, config.star_count);

    //todo: #bevy_question - I've tried a bunch of different implementations
    //                      but it all comes down to calling iter() when there are
    //                      thousands of entities - it slows things down enough to
    // affect the frame                      rate in dev - more than half a ms
    // for iterating 3000 entities                      that seems bonkers to
    // me...a lot of the time seems to be in the destructuring...

    // this takes about 70-80K ns in dev and in release it's many times faster
    // which becomes negligible - so for now, i guess live with it -
    // still don't understand why it's so slow
    // it's more like 500K if we use the iter directly and randomize as
    // the destructuring in the loop eats up 90% of the loop time
    // this pre-filtering avoids that cost - i don't know what is the difference
    // of collecting into a Vec vs. destructuring in the for loop - but it's a LOT
    // slower
    let all_stars: Vec<(Entity, &MeshMaterial3d<StandardMaterial>)> = stars.iter().collect();
    let filtered_stars = extract_elements_at_indices(&all_stars, &indices);

    let mut rng = rand::rng();

    for (entity, material_handle) in filtered_stars {
        if let Some(material) = materials.get(material_handle) {
            let original_emissive = Vec4::new(
                material.emissive.red,
                material.emissive.green,
                material.emissive.blue,
                material.emissive.alpha,
            );
            let intensity = rng.random_range(config.twinkle_intensity.start..config.twinkle_intensity.end);
            let target_emissive = original_emissive * intensity;

            let duration = rng.random_range(config.twinkle_duration.start..config.twinkle_duration.end);

            commands.entity(entity).insert(Twinkling {
                original_emissive,
                target_emissive,
                twinkle_timer: Timer::from_seconds(duration, TimerMode::Once),
            });
        }
    }
}

fn update_twinkling(
    mut commands: Commands,
    time: Res<Time>,
    mut stars: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &mut Twinkling)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, material_handle, mut twinkling) in stars.iter_mut() {
        twinkling.twinkle_timer.tick(time.delta());

        if let Some(material) = materials.get_mut(material_handle) {
            let progress =
                twinkling.twinkle_timer.elapsed_secs() / twinkling.twinkle_timer.duration().as_secs_f32();
            // .5 -> brighten until halfway and then dim back
            // .2 in the lerp -> used to make sure full range of the lerp function is used
            // in each                   half of the animation - so we go from
            // 0..1 in each half
            let new_emissive = if progress < 0.5 {
                twinkling
                    .original_emissive
                    .lerp(twinkling.target_emissive, progress * 2.0)
            } else {
                twinkling
                    .target_emissive
                    .lerp(twinkling.original_emissive, (progress - 0.5) * 2.0)
            };
            material.emissive =
                LinearRgba::new(new_emissive.x, new_emissive.y, new_emissive.z, new_emissive.w);
        }

        if twinkling.twinkle_timer.finished() {
            commands.entity(entity).remove::<Twinkling>();
        }
    }
}
