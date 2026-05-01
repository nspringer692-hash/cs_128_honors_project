use bevy::prelude::*;

// Store all components for Ferroforge here

// Each gate will have inputs and outputs
#[derive(Component)]
pub struct Port {
    pub port_id: i32, // 0 if left, 1 if right
    pub is_output: bool, // is this port an input or output?
}

#[derive(Component)]
pub struct Wire {
    from: Entity, // Where does this wire start
    to: Entity, // ...and where does it end?
}

// All components used for dragging stuff
#[derive(Component)]
pub struct Draggable;

#[derive(Resource, Default)]
pub struct DragState {
    pub entity: Option<Entity>,
}