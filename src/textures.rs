use bevy::prelude::*;

#[derive(Resource)]
pub struct Textures {
    pub nand_gate: Handle<Image>,
    pub nor_gate: Handle<Image>,
    pub not_gate: Handle<Image>,
    pub and_gate: Handle<Image>,
    pub or_gate: Handle<Image>,
    pub xor_gate: Handle<Image>,
    pub xnor_gate: Handle<Image>,
    pub port: Handle<Image>,
    pub wire: Handle<Image>,
    pub output_a: Handle<Image>,
    pub output_b: Handle<Image>,
    pub input_f: Handle<Image>,
}

// Load the textures outside of setup
pub fn load_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Textures {
        nand_gate: asset_server.load("textures/nand.png"),
        nor_gate: asset_server.load("textures/nor.png"),
        not_gate: asset_server.load("textures/not.png"),
        and_gate: asset_server.load("textures/and.png"),
        or_gate: asset_server.load("textures/or.png"),
        xor_gate: asset_server.load("textures/xor.png"),
        xnor_gate: asset_server.load("textures/xnor.png"),
        port: asset_server.load("textures/input.png"),
        wire: asset_server.load("textures/connector.png"),
        output_a: asset_server.load("textures/static_input_a.png"),
        output_b: asset_server.load("textures/static_input_b.png"),
        input_f: asset_server.load("textures/static_input_f.png"),
    });
}