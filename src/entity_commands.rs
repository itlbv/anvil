use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::Position;
use crate::{behaviors, BehaviorList, Knowledge};
use hecs::World as ComponentRegistry;
use sdl2::ttf::init;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    MoveToPosition { x: f32, y: f32 },
    RemoveFromMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCommand {
    #[serde(with = "crate::entity_serde")]
    entity: Entity,
    kind: CommandType,
}

impl EntityCommand {
    fn new(entity: Entity, kind: CommandType) -> Self {
        Self { entity, kind }
    }

    fn move_to(entity: Entity, x: f32, y: f32) -> Self {
        Self {
            entity,
            kind: CommandType::MoveToPosition { x, y },
        }
    }

    fn remove_from_map(entity: Entity) -> Self {
        Self {
            entity,
            kind: CommandType::RemoveFromMap,
        }
    }
}

#[derive(Copy, Clone)]
pub struct CommandMeta {
    pub source: &'static str,
    pub file: &'static str,
    pub line: u32,
}
impl Default for CommandMeta {
    fn default() -> Self {
        Self {
            source: "unknown source",
            file: "unknown file",
            line: 0,
        }
    }
}

#[track_caller]
#[inline]
pub fn push_with_meta(
    commands: &mut Vec<EntityCommand>,
    cmd: EntityCommand,
    mut meta: CommandMeta,
) {
    // Autofill context if not provided.
    if meta.source == "unknown source" {
        meta.source = module_path!();
    }
    if meta.file == "unknown file" {
        let loc = std::panic::Location::caller();
        meta.file = loc.file();
        meta.line = loc.line();
    }

    commands.push(cmd);
}

#[track_caller]
#[inline]
pub fn push_new_command(out: &mut Vec<EntityCommand>, entity: Entity, kind: CommandType) {
    push_with_meta(
        out,
        EntityCommand::new(entity, kind),
        CommandMeta::default(),
    );
}

pub mod emit {
    use super::*;

    #[track_caller]
    #[inline]
    pub fn move_to(commands: &mut Vec<EntityCommand>, entity: Entity, x: f32, y: f32) {
        push_with_meta(
            commands,
            EntityCommand::move_to(entity, x, y),
            CommandMeta::default(),
        );
    }

    #[track_caller]
    #[inline]
    pub fn remove_from_map(commands: &mut Vec<EntityCommand>, entity: Entity) {
        push_with_meta(
            commands,
            EntityCommand::remove_from_map(entity),
            CommandMeta::default(),
        );
    }
}

pub fn process_commands(
    commands: &mut Vec<EntityCommand>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    behaviours: &mut HashMap<Entity, BehaviorList>,
    registry: &mut ComponentRegistry,
) {
    while let Some(cmd) = commands.pop() {
        match cmd.kind {
            CommandType::MoveToPosition { x, y } => {
                let entity_behaviours = behaviours
                    .get_mut(&cmd.entity)
                    .expect("behaviours missing for entity");
                entity_behaviours.insert(0, behaviors::move_to_position());

                let knowledge = knowledges
                    .get_mut(&cmd.entity)
                    .expect("knowledge missing for entity");
                knowledge.destination_x = x;
                knowledge.destination_y = y;
            }
            CommandType::RemoveFromMap => {
                registry
                    .remove_one::<Position>(cmd.entity)
                    .expect("failed to remove Position component");
            }
        }
    }
}

pub fn resolve_commands(cmds: &mut Vec<EntityCommand>) {
    use std::collections::HashMap;

    // Keep RemoveFromMap in original order.
    let mut removes: Vec<EntityCommand> = Vec::new();

    // For MoveToPosition: last-wins per entity.
    let mut last_move: HashMap<u64, EntityCommand> = HashMap::new();

    for cmd in cmds.drain(..) {
        match cmd.kind {
            CommandType::RemoveFromMap => removes.push(cmd),
            CommandType::MoveToPosition { .. } => {
                let id = cmd.entity.to_bits().get();
                last_move.insert(id, cmd); // overwrite -> last wins
            }
        }
    }

    // Deterministic rebuild:
    // 1) original-order removes
    cmds.extend(removes.into_iter());

    // 2) moves sorted by entity id to avoid HashMap iteration nondeterminism
    let mut moves: Vec<_> = last_move.into_values().collect();
    moves.sort_by_key(|c| c.entity.to_bits().get());
    cmds.extend(moves.into_iter());
}
