use bevy::{ecs::system::EntityCommands, math::vec2, prelude::*};

mod slide;
pub use slide::*;

/// Trait for adding transition components to entities as they go through their lifecycle.
pub trait Transition: Copy {
    /// Called for entities that have been newly inserted.
    /// When `root` is `true`, the entity is being inserted in an existing parent,
    /// otherwise the parent is also newly inserted.
    fn insert(&self, commands: &mut EntityCommands, root: bool);

    /// Called for entities that should be removed.
    /// The `Transition` implementation is responsible for calling `despawn_recursive` either
    /// immediately or after completing an animation.
    fn remove(&self, commands: EntityCommands);
}

/// Default transition behavior without any animations.
#[derive(Clone, Copy)]
pub struct DefaultTransition;

impl Transition for DefaultTransition {
    fn insert(&self, _commands: &mut EntityCommands, _root: bool) {}

    fn remove(&self, commands: EntityCommands) {
        commands.despawn_recursive();
    }
}

pub fn get_item_anchor(style: &Style) -> Vec2 {
    match style.display {
        Display::Flex => (),
        Display::None => return default(),
    }
    match (style.flex_direction, style.align_items) {
        (FlexDirection::Row, AlignItems::Start) => vec2(-1.0, -1.0),
        (FlexDirection::Row, AlignItems::End) => vec2(-1.0, 1.0),
        (FlexDirection::Row, AlignItems::FlexStart) => vec2(-1.0, -1.0),
        (FlexDirection::Row, AlignItems::FlexEnd) => vec2(-1.0, 1.0),
        (FlexDirection::Row, AlignItems::Center) => vec2(-1.0, 0.0),
        (FlexDirection::Row, AlignItems::Baseline) => vec2(-1.0, 0.0),
        (FlexDirection::Row, AlignItems::Stretch) => vec2(-1.0, 0.0),
        (FlexDirection::Column, AlignItems::Start) => vec2(-1.0, -1.0),
        (FlexDirection::Column, AlignItems::End) => vec2(1.0, -1.0),
        (FlexDirection::Column, AlignItems::FlexStart) => vec2(-1.0, -1.0),
        (FlexDirection::Column, AlignItems::FlexEnd) => vec2(1.0, -1.0),
        (FlexDirection::Column, AlignItems::Center) => vec2(0.0, -1.0),
        (FlexDirection::Column, AlignItems::Baseline) => vec2(0.0, -1.0),
        (FlexDirection::Column, AlignItems::Stretch) => vec2(0.0, -1.0),
        (FlexDirection::RowReverse, AlignItems::Start) => vec2(1.0, -1.0),
        (FlexDirection::RowReverse, AlignItems::End) => vec2(1.0, 1.0),
        (FlexDirection::RowReverse, AlignItems::FlexStart) => vec2(1.0, 1.0),
        (FlexDirection::RowReverse, AlignItems::FlexEnd) => vec2(1.0, -1.0),
        (FlexDirection::RowReverse, AlignItems::Center) => vec2(1.0, 0.0),
        (FlexDirection::RowReverse, AlignItems::Baseline) => vec2(1.0, 0.0),
        (FlexDirection::RowReverse, AlignItems::Stretch) => vec2(1.0, 0.0),
        (FlexDirection::ColumnReverse, AlignItems::Start) => vec2(-1.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::End) => vec2(1.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::FlexStart) => vec2(1.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::FlexEnd) => vec2(-1.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::Center) => vec2(0.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::Baseline) => vec2(0.0, 1.0),
        (FlexDirection::ColumnReverse, AlignItems::Stretch) => vec2(0.0, 1.0),
    }
}
