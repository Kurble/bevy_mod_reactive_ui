use bevy::{ecs::system::EntityCommands, prelude::*};

mod slide;
pub use slide::*;

pub trait Transition: Copy {
    fn insert(&self, commands: EntityCommands) -> Entity;

    fn update(&self, commands: EntityCommands, layout: Rect);

    fn remove(&self, commands: EntityCommands);
}

#[derive(Clone, Copy)]
pub struct DefaultTransition;

impl Transition for DefaultTransition {
    fn insert(&self, commands: EntityCommands) -> Entity {
        commands.id()
    }

    fn update(&self, _: EntityCommands, _: Rect) {}

    fn remove(&self, commands: EntityCommands) {
        commands.despawn_recursive();
    }
}
