use hecs::Entity;
use std::collections::HashMap;

use crate::components::Position;
use crate::{behaviors, BehaviorList, Knowledge};
use hecs::World as ComponentRegistry;

#[derive(PartialEq)]
pub enum EntityCommandType {
    MoveToPosition,
    RemoveFromMap,
}

pub struct EntityCommand {
    pub entity: Entity,
    pub event_type: EntityCommandType,
    pub param: HashMap<String, String>,
}

pub fn process_entity_commands(
    entity_commands: &mut Vec<EntityCommand>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    behaviors: &mut HashMap<Entity, BehaviorList>,
    registry: &mut ComponentRegistry,
) {
    while !entity_commands.is_empty() {
        // break;
        let entity_event = entity_commands.pop().unwrap();

        match entity_event.event_type {
            EntityCommandType::MoveToPosition => {
                // dispatch MoveToBehavior for an entity
                let mut entity_behaviors = behaviors.get_mut(&entity_event.entity).unwrap();
                entity_behaviors.insert(0, behaviors::move_to_position());
                // add info to knowledge
                let mut knowledge = knowledges.get_mut(&entity_event.entity).unwrap();
                knowledge.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                knowledge.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
            }
            EntityCommandType::RemoveFromMap => {
                registry
                    .remove_one::<Position>(entity_event.entity)
                    .expect("TODO: panic message");
            }
        }
    }
}
