use std::time::Duration;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_mod_reactive_ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Demo".into(),
                resolution: (400., 300.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShadowScenePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, some_ui_system)
        .insert_resource(Counter { value: 0 })
        .run();
}

#[derive(Resource)]
struct Counter {
    value: i32,
}

#[derive(Resource)]
struct LoadedAssets {
    text_style: TextStyle,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(LoadedAssets {
        text_style: TextStyle {
            font: asset_server.load("font.ttf"),
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    });
}

fn some_ui_system(mut shadow: ShadowScene, state: Res<Counter>, assets: Res<LoadedAssets>) {
    let slide = SlideTransition {
        direction: Vec2::new(-80.0, 0.0),
        duration: Duration::from_millis(300),
    };
    shadow.update_with_transition(&slide, |shadow| {
        shadow.spawn(id!(), vbox).with(|shadow| {
            if state.value < 10 {
                shadow
                    .spawn(id!(), || button().on_click(on_up).on_event(on_up_key))
                    .spawn(id!(), || label("up", assets.text_style.clone()));
            }
            shadow.spawn_dyn(id!(), state.is_changed(), || {
                label(format!("count: {}", state.value), assets.text_style.clone())
            });
            if state.value > 0 {
                shadow
                    .spawn(id!(), || button().on_click(on_down).on_event(on_down_key))
                    .spawn(id!(), || label("down", assets.text_style.clone()));
            }
        });
    });
}

fn on_up(mut state: ResMut<Counter>) {
    state.value += 1;
}

fn on_down(mut state: ResMut<Counter>) {
    state.value -= 1;
}

fn on_up_key(In(key): In<KeyboardInput>, mut state: ResMut<Counter>) {
    let Some(KeyCode::Up) = key.key_code else { return; };
    let ButtonState::Pressed = key.state else { return; };
    state.value += 10;
}

fn on_down_key(In(key): In<KeyboardInput>, mut state: ResMut<Counter>) {
    let Some(KeyCode::Down) = key.key_code else { return; };
    let ButtonState::Pressed = key.state else { return; };
    state.value -= 10;
}

fn vbox() -> impl Bundle {
    NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_self: AlignSelf::Center,
            margin: UiRect::horizontal(Val::Auto),
            padding: UiRect::all(Val::Px(32.0)),
            width: Val::Px(256.0),
            height: Val::Px(256.0),
            ..default()
        },
        background_color: BackgroundColor(Color::GRAY),
        ..default()
    }
}

fn button() -> impl Bundle {
    ButtonBundle::default()
}

fn label(text: impl Into<String>, style: TextStyle) -> impl Bundle {
    TextBundle {
        text: Text::from_section(text, style),
        ..default()
    }
}
