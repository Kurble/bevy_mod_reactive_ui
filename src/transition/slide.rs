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
    Resident(Vec3, Vec2),
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
    mut query: Query<(Entity, &mut Slide, &Node, &mut Transform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Some(time) = time.last_update() else { return; };

    for (entity, mut transition, node, mut transform) in query.iter_mut() {
        match transition.phase {
            Phase::Inserted(start, root) => {
                let t = if let Some(duration) = time.checked_duration_since(start) {
                    (duration.as_secs_f32() / transition.duration.as_secs_f32()).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                if t < 1.0  && root {
                    transform.translation += (transition.direction * (1.0 - t)).extend(0.0);
                    transform.scale = Vec3::new(t, t, 1.0);
                } else {
                    transform.scale = Vec3::ONE;
                    transition.phase = Phase::Resident(transform.translation, node.size());
                }
            }
            Phase::Resident(ref mut pos, ref mut size) => {
                let size_difference = node.size() - *size;
                *size = node.size();

                *pos += size_difference.extend(0.0) * 0.5;
                *pos = pos.lerp(transform.translation, 0.2);
                transform.translation = *pos;
            },
            Phase::Removed(start) => {
                let t = if let Some(duration) = time.checked_duration_since(start) {
                    (duration.as_secs_f32() / transition.duration.as_secs_f32()).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                if t < 1.0 {
                    transform.translation += (transition.direction * t).extend(0.0);
                    transform.scale = Vec3::new(1.0 - t, 1.0 - t, 1.0);
                } else {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
