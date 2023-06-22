use bevy::{
    ecs::{query::QueryEntityError, system::EntityCommands},
    math::vec2,
    prelude::*,
};

mod slide;
pub use slide::*;

/// Trait for adding transition components to entities as they go through their lifecycle.
pub trait Transition {
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

#[derive(Clone, Copy)]
pub struct Placement {
    position: Vec2,
    subject_size: Vec2,
    parent_size: Vec2,
}

impl Placement {
    pub fn new(
        query: &Query<(&Node, &Style)>,
        parent: Entity,
        subject: Entity,
        position: Vec2,
    ) -> Result<Self, QueryEntityError> {
        let (parent_node, _parent_style) = query.get(parent)?;
        let (subject_node, _subject_style) = query.get(subject)?;
        Ok(Self {
            parent_size: parent_node.size(),
            subject_size: subject_node.size(),
            position,
        })
    }

    pub fn preserve_anchor(
        &mut self,
        query: &Query<(&Node, &Style)>,
        parent: Entity,
        subject: Entity,
    ) -> Result<(), QueryEntityError> {
        let (parent_node, parent_style) = query.get(parent)?;
        let (subject_node, subject_style) = query.get(subject)?;
        let anchor = get_anchor(parent_style, subject_style);

        let subject_anchor_point = get_anchor_point(self.position, self.subject_size, anchor);
        let parent_anchor_point = get_anchor_point(Vec2::ZERO, self.parent_size, anchor);
        let anchor_offset = subject_anchor_point - parent_anchor_point;

        self.subject_size = subject_node.size();
        self.parent_size = parent_node.size();
        let subject_anchor_point = get_anchor_point(Vec2::ZERO, self.subject_size, anchor);
        let parent_anchor_point = get_anchor_point(Vec2::ZERO, self.parent_size, anchor);
        self.position = parent_anchor_point + anchor_offset - subject_anchor_point;

        Ok(())
    }
}

fn get_anchor(parent: &Style, subject: &Style) -> Vec2 {
    match parent.display {
        Display::Flex => (),
        Display::None => return default(),
    }

    let reverse = match parent.flex_direction {
        FlexDirection::Row | FlexDirection::Column => -0.5,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse => 0.5,
    };

    let cross = match subject.align_self {
        AlignSelf::Auto => match parent.align_items {
            AlignItems::Start => 0.0,
            AlignItems::End => 1.0,
            AlignItems::FlexStart => 0.5 - reverse,
            AlignItems::FlexEnd => 0.5 + reverse,
            AlignItems::Center => 0.5,
            AlignItems::Baseline => 0.5,
            AlignItems::Stretch => 0.5,
        },
        AlignSelf::Start => 0.0,
        AlignSelf::End => 1.0,
        AlignSelf::FlexStart => 0.5 - reverse,
        AlignSelf::FlexEnd => 0.5 + reverse,
        AlignSelf::Center => 0.5,
        AlignSelf::Baseline => 0.5,
        AlignSelf::Stretch => 0.5,
    };

    match parent.flex_direction {
        FlexDirection::Row => vec2(0.0, cross),
        FlexDirection::Column => vec2(cross, 0.0),
        FlexDirection::RowReverse => vec2(1.0, cross),
        FlexDirection::ColumnReverse => vec2(cross, 1.0),
    }
}

fn get_anchor_point(pos: Vec2, size: Vec2, anchor: Vec2) -> Vec2 {
    vec2(
        pos.x - size.x * 0.5 + size.x * anchor.x,
        pos.y - size.y * 0.5 + size.y * anchor.y,
    )
}
