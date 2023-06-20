use bevy::utils::{Duration, Instant};

use super::*;

#[derive(Clone, Copy)]
pub struct SlideTransition {
    pub direction: Vec2,
    pub duration: Duration,
}

enum TransitionPhase {
    Inserted,
    Updated(Rect),
    Removed,
}

#[derive(Component)]
pub(crate) struct TransitionComponent {
    phase: TransitionPhase,
    start: Instant,
    duration: Duration,
    direction: Vec2,
}

impl Transition for SlideTransition {
    fn insert(&self, mut commands: EntityCommands) -> Entity {
        commands
            .insert(TransitionComponent {
                phase: TransitionPhase::Inserted,
                start: Instant::now(),
                duration: self.duration,
                direction: self.direction,
            })
            .id()
    }

    fn update(&self, mut commands: EntityCommands, layout: Rect) {
        commands.insert(TransitionComponent {
            phase: TransitionPhase::Updated(layout),
            start: Instant::now(),
            duration: self.duration,
            direction: self.direction,
        });
    }

    fn remove(&self, mut commands: EntityCommands) {
        commands.insert(TransitionComponent {
            phase: TransitionPhase::Removed,
            start: Instant::now(),
            duration: self.duration,
            direction: self.direction,
        });
    }
}

pub(crate) fn slide_transition_system(
    mut query: Query<(Entity, &TransitionComponent, &Node, &mut Transform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Some(time) = time.last_update() else { return; };

    for (entity, transition, _node, mut transform) in query.iter_mut() {
        let t = if let Some(duration) = time.checked_duration_since(transition.start) {
            (duration.as_secs_f32() / transition.duration.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if t >= 1.0 {
            match transition.phase {
                TransitionPhase::Inserted | TransitionPhase::Updated(_) => {
                    transform.scale = Vec3::ONE;
                    commands.entity(entity).remove::<TransitionComponent>();
                    continue;
                }
                TransitionPhase::Removed => {
                    commands.entity(entity).despawn_recursive();
                    continue;
                }
            }
        }

        match transition.phase {
            TransitionPhase::Inserted => {
                transform.translation += (transition.direction * (1.0 - t)).extend(0.0);
                transform.scale = Vec3::new(t, t, 1.0);
            }
            TransitionPhase::Updated(from) => {
                //let current_layout =
                //Rect::from_center_size(transform.translation.truncate(), node.size());
                //if current_layout != from {
                transform.translation = from
                    .center()
                    .extend(transform.translation.z)
                    .lerp(transform.translation, t);
                //} else {
                //commands.entity(entity).remove::<TransitionComponent>();
                //}
            }
            TransitionPhase::Removed => {
                transform.translation += (transition.direction * t).extend(0.0);
                transform.scale = Vec3::new(1.0 - t, 1.0 - t, 1.0);
            }
        };
    }
}
