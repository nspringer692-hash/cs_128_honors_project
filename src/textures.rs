use bevy::prelude::*;

#[derive(Resource)]
pub struct Textures {
    pub gate: Handle<Image>,
    pub port: Handle<Image>,
    pub wire: Handle<Image>,
}

// Load the textures outside of setup
pub fn load_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Textures {
        gate: asset_server.load("textures/placeholder2.png"),
        port: asset_server.load("textures/input.png"),
        wire: asset_server.load("textures/connector.png"),
    });
}