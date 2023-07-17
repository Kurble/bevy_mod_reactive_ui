mod base_handler;
mod event_handler;
mod interaction_handler;
mod shadow;
mod transition;

pub use event_handler::{EventHandler, SetEventHandler};
use interaction_handler::{
    make_interaction_handler_system, OnClick, OnClickEnd, OnHover, OnHoverEnd,
};
pub use interaction_handler::{InteractionHandler, SetInteractionHandler};
pub use shadow::{Shadow, ShadowScene};
pub use transition::*;

use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
    ui::UiSystem,
};

pub struct ShadowScenePlugin;

impl Plugin for ShadowScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                make_interaction_handler_system::<OnClick>(),
                make_interaction_handler_system::<OnClickEnd>(),
                make_interaction_handler_system::<OnHover>(),
                make_interaction_handler_system::<OnHoverEnd>(),
                event_handler::make_event_handler_system::<KeyboardInput>(),
                event_handler::make_event_handler_system::<GamepadEvent>(),
            ),
        );

        app.add_systems(PostUpdate, slide_transition_system.after(UiSystem::Layout));
    }
}
