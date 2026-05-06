use bevy::prelude::*;

// Store all components for Ferroforge here

// Each gate will have inputs and outputs
#[derive(Component)]
pub struct Port {
    pub port_id: i32, // 0 if left, 1 if right
    pub is_output: bool, // is this port an input or output?
    pub identifier: i32,
}

#[derive(Component)]
pub struct Wire {
    pub from: Entity, // Where does this wire start
    pub to: Entity, // ...and where does it end?
}

#[derive(Component)]
pub struct WireSegment;

// Used to track which port is clicked on
#[derive(Resource, Default)]
pub struct ConnectionState {
    pub selected_input: Option<Entity>,
    pub just_selected: bool,
}

// All components used for dragging stuff
#[derive(Component, Default)]
pub struct Draggable;

#[derive(Resource, Default)]
pub struct DragState {
    pub entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct ActiveLevel(pub Option<u32>);