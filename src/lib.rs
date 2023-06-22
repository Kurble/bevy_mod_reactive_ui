mod base_handler;
mod event_handler;
mod interaction_handler;
mod mount;
mod transition;

pub use event_handler::{EventHandler, SetEventHandler};
use interaction_handler::{
    make_interaction_handler_system, OnClick, OnClickEnd, OnHover, OnHoverEnd,
};
pub use interaction_handler::{InteractionHandler, SetInteractionHandler};
pub use mount::{Mount, Fragment};
pub use transition::*;

use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
    ui::UiSystem,
};

pub struct ReactivePlugin;

impl Plugin for ReactivePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(make_interaction_handler_system::<OnClick>());
        app.add_system(make_interaction_handler_system::<OnClickEnd>());
        app.add_system(make_interaction_handler_system::<OnHover>());
        app.add_system(make_interaction_handler_system::<OnHoverEnd>());
        app.add_system(event_handler::make_event_handler_system::<KeyboardInput>());
        app.add_system(event_handler::make_event_handler_system::<GamepadEvent>());
        app.add_system(
            slide_transition_system
                .in_set(UiSystem::Flex)
                .after(bevy::ui::flex_node_system),
        );
    }
}
