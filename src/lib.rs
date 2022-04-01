#![deny(missing_docs)]

//! Add a diagnostics overlay (with an FPS counter) in Bevy.
//!
//! This crate provides a Bevy [plugin](ScreenDiagsPlugin) to add the diagnostics overlay.
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    utils::Duration,
};

const FONT_SIZE: f32 = 32.0;
const FONT_COLOR: Color = Color::RED;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

/// A plugin that draws diagnostics on-screen with Bevy UI.
///
/// Use our [marker struct](ScreenDiagsTimer) to manage the FPS counter.
pub struct ScreenDiagsPlugin;

impl Plugin for ScreenDiagsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup)
            .add_system(update);
    }
}

/// The marker component for our FPS update interval timer.
///
/// To disable the FPS counter, write a query for a [Timer](bevy::prelude::Timer) filtered by this
/// struct and pause the timer. Unpause the timer to re-enable the counter.
#[derive(Component)]
pub struct ScreenDiagsTimer {
    text_entity: Option<Entity>,
}

#[derive(Component)]
struct ScreenDiagsText;

fn update(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut timer_query: Query<(&mut ScreenDiagsTimer, &mut Timer)>,
    mut text_query: Query<&mut Text, With<ScreenDiagsText>>,
) {
    let (mut marker, mut timer) = timer_query.single_mut();

    match marker.text_entity {
        // Overlay is disabled and has already been despawned - do nothing.
        None if timer.paused() => {}

        // Overlay has just been enabled but doesn't exist yet - should spawn it.
        None => {
            marker.text_entity = Some(spawn_text(
                &mut commands,
                asset_server,
                extract_fps(diagnostics).map(|fps| {
                    let mut buffer = String::new();
                    format_fps(&mut buffer, fps);
                    buffer
                }),
            ));
        }

        // Overlay has just been disabled, but still exists - should despawn it.
        Some(text_entity) if timer.paused() => {
            commands.entity(text_entity).despawn_recursive();
            marker.text_entity.take();
        }

        // Overlay is enabled and exists, but UPDATE_INTERVAL hasn't passed yet - do nothing.
        Some(_) if !timer.tick(time.delta()).just_finished() => {}

        // Overlay is enabled and exists, and UPDATE_INTERVAL has passed - try to update it.
        Some(_) => {
            if let Some(fps) = extract_fps(diagnostics) {
                let mut text = text_query.single_mut();
                format_fps(&mut text.sections[1].value, fps);
            }
        }
    }
}

fn extract_fps(diagnostics: Res<Diagnostics>) -> Option<f64> {
    diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .map(|fps| fps.average().unwrap_or_default())
}

fn format_fps(buffer: &mut String, fps: f64) {
    *buffer = format!("{:.0}", fps);
}

/// Set up the UI camera, the text element and, attached to it, the plugin state.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let entity = spawn_text(&mut commands, asset_server, None);
    commands.spawn_bundle((
        ScreenDiagsTimer {
            text_entity: Some(entity),
        },
        Timer::new(UPDATE_INTERVAL, true),
    ));
}

fn spawn_text(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    fps: Option<String>,
) -> Entity {
    let handle = asset_server.load("fonts/screen-diags-font.ttf");
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: handle.clone(),
                            font_size: FONT_SIZE,
                            color: FONT_COLOR,
                        },
                    },
                    TextSection {
                        value: fps.unwrap_or_else(|| "...".to_string()),
                        style: TextStyle {
                            font: handle,
                            font_size: FONT_SIZE,
                            color: FONT_COLOR,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScreenDiagsText)
        .id()
}
