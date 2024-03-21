pub mod car;
pub mod track;
pub mod menu;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins, 
        menu::MenuPlugin, 
        WorldInspectorPlugin::new()
    ));
    app.add_systems(Startup, spawn_scene);
    app.run();
}

pub fn spawn_scene(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
}
