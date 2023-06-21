use bevy::utils::{Duration, Instant};

use super::*;

#[derive(Clone, Copy)]
pub struct SlideTransition {
    pub direction: Vec2,
    pub duration: Duration,
}

#[derive(Clone, Copy)]
enum Phase {
    Inserted(Instant, bool),
    Resident(Placement),
    Removed(Instant),
}

#[derive(Component)]
pub(crate) struct Slide {
    phase: Phase,
    duration: Duration,
    direction: Vec2,
}

impl Transition for SlideTransition {
    fn insert(&self, commands: &mut EntityCommands, root: bool) {
        commands.insert(Slide {
            phase: Phase::Inserted(Instant::now(), root),
            duration: self.duration,
            direction: self.direction,
        });
    }

    fn remove(&self, mut commands: EntityCommands) {
        commands.insert(Slide {
            phase: Phase::Removed(Instant::now()),
            duration: self.duration,
            direction: self.direction,
        });
    }
}

pub(crate) fn slide_transition_system(
    mut query: Query<(Entity, Option<&Parent>, &mut Slide, &mut Transform)>,
    parent_query: Query<(&Node, &Style)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Some(time) = time.last_update() else { return; };

    for (entity, parent, mut slide, mut transform) in query.iter_mut() {
        match slide.phase {
            Phase::Inserted(start, root) => {
                let t = if let Some(duration) = time.checked_duration_since(start) {
                    (duration.as_secs_f32() / slide.duration.as_secs_f32()).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                if t < 1.0 && root {
                    transform.translation += (slide.direction * (1.0 - t)).extend(0.0);
                    transform.scale = Vec3::new(t, t, 1.0);
                } else if let Some(parent) = parent {
                    transform.scale = Vec3::ONE;
                    slide.phase = Phase::Resident(
                        Placement::new(
                            &parent_query,
                            parent.get(),
                            entity,
                            transform.translation.truncate(),
                        )
                        .unwrap(),
                    );
                } else {
                    commands.entity(entity).remove::<Slide>();
                }
            }
            Phase::Resident(ref mut placement) => {
                if let Some(parent) = parent {
                    placement
                        .preserve_anchor(&parent_query, parent.get(), entity)
                        .ok();
                    placement.position = placement
                        .position
                        .lerp(transform.translation.truncate(), 0.1);
                    transform.translation = placement.position.extend(transform.translation.z);
                }
            }
            Phase::Removed(start) => {
                let t = if let Some(duration) = time.checked_duration_since(start) {
                    (duration.as_secs_f32() / slide.duration.as_secs_f32()).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                if t < 1.0 {
                    transform.translation += (slide.direction * t).extend(0.0);
                    transform.scale = Vec3::new(1.0 - t, 1.0 - t, 1.0);
                } else {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
