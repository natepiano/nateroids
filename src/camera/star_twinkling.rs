use std::collections::HashSet;

use bevy::prelude::*;
use rand::RngExt;

use super::constants::STAR_TWINKLE_HALF_SCALE;
use super::constants::STAR_TWINKLE_MIDPOINT;
use super::settings::StarSettings;
use super::stars::Star;

pub(super) struct StarTwinklingPlugin;

impl Plugin for StarTwinklingPlugin {
    fn build(&self, app: &mut App) {
        let start_twinkling_timer_duration = StarSettings::default().twinkle.delay;
        app.insert_resource(StartTwinklingTimer {
            timer: Timer::from_seconds(start_twinkling_timer_duration, TimerMode::Repeating),
        })
        .add_systems(Update, (start_twinkling, update_twinkling));
    }
}

#[derive(Component)]
struct Twinkling {
    original_emissive: Vec4,
    target_emissive:   Vec4,
    twinkle_timer:     Timer,
}

#[derive(Resource)]
struct StartTwinklingTimer {
    timer: Timer,
}

fn should_start_twinkling(start_timer: &mut StartTwinklingTimer, time: &Time) -> bool {
    start_timer.timer.tick(time.delta());
    start_timer.timer.just_finished()
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
    indices
        .iter()
        .filter_map(|&i| vec.get(i).cloned())
        .collect()
}

// yeah - but how can the query below be much simpler?
fn start_twinkling(
    mut commands: Commands,
    star_settings: Res<StarSettings>,
    stars: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (With<Star>, Without<Twinkling>)>,
    materials: Res<Assets<StandardMaterial>>,
    mut start_timer: ResMut<StartTwinklingTimer>,
    time: Res<Time>,
) {
    if !should_start_twinkling(&mut start_timer, &time) {
        return;
    }

    let all_stars: Vec<(Entity, &MeshMaterial3d<StandardMaterial>)> = stars.iter().collect();
    let twinkle_count = star_settings
        .twinkle
        .choose_multiple_count
        .min(all_stars.len());
    let indices = get_random_indices(twinkle_count, all_stars.len());

    // Snapshot the untwinkling stars once so we can sample a bounded subset by index.
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
            let intensity = rng.random_range(
                star_settings.twinkle.intensity.start..star_settings.twinkle.intensity.end,
            );
            let target_emissive = original_emissive * intensity;

            let duration = rng.random_range(
                star_settings.twinkle.duration.start..star_settings.twinkle.duration.end,
            );

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
    for (entity, material_handle, mut twinkling) in &mut stars {
        twinkling.twinkle_timer.tick(time.delta());

        if let Some(material) = materials.get_mut(material_handle) {
            let progress = twinkling.twinkle_timer.elapsed_secs()
                / twinkling.twinkle_timer.duration().as_secs_f32();
            let new_emissive = if progress < STAR_TWINKLE_MIDPOINT {
                twinkling.original_emissive.lerp(
                    twinkling.target_emissive,
                    progress * STAR_TWINKLE_HALF_SCALE,
                )
            } else {
                twinkling.target_emissive.lerp(
                    twinkling.original_emissive,
                    (progress - STAR_TWINKLE_MIDPOINT) * STAR_TWINKLE_HALF_SCALE,
                )
            };
            material.emissive = LinearRgba::new(
                new_emissive.x,
                new_emissive.y,
                new_emissive.z,
                new_emissive.w,
            );
        }

        if twinkling.twinkle_timer.is_finished() {
            commands.entity(entity).remove::<Twinkling>();
        }
    }
}
