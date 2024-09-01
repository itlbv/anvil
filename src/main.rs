mod behaviors;
mod btree;
mod components;
mod input_controller;
mod systems;
mod util;
mod window;

use crate::btree::BehaviorTreeNode;
use crate::components::StateType::{IDLE, MOVE};
use crate::components::{Food, Hunger, Movement, Position, Shape, State, StateType};
use crate::input_controller::InputController;
use crate::systems::{choose_behaviors, draw, hunger, movement, run_behaviors};
use crate::window::Window;
use crate::EntityCommandType::MoveToPosition;
use hecs::Entity;
use hecs::World as ComponentRegistry;
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

type BehaviorList = Vec<Box<dyn BehaviorTreeNode>>;

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
}

#[derive(PartialEq)]
enum EntityCommandType {
    MoveToPosition,
    RemoveFromMap,
}

struct EntityCommand {
    entity: Entity,
    event_type: EntityCommandType,
    param: HashMap<String, String>,
}

struct Knowledge {
    own_id: Entity,
    target: Option<Entity>,
    destination_x: f32,
    destination_y: f32,
    inventory: Vec<Entity>,
    map: HashMap<String, String>,
}

trait EntityTask {
    fn run(&mut self, entity: Entity, world: &mut ComponentRegistry);
}

struct MoveTask {}

impl EntityTask for MoveTask {
    fn run(&mut self, entity: Entity, world: &mut ComponentRegistry) {
        // check if close to target already

        // when task finished tell to continue behavior
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut input_controller = InputController::new(&sdl_context);

    let mut properties = Properties {
        quit: false,
        selected_entity: None,
    };

    let mut registry = ComponentRegistry::new();

    let mut rand = rand::thread_rng();
    let food_to_spawn = (0..6).map(|_| {
        let pos = Position::new(rand.gen_range(2..10) as f32, rand.gen_range(2..10) as f32);
        let shape = Shape::new(0.2, 0.2, (150, 40, 40, 255));
        let food = Food {};
        (pos, shape, food)
    });
    registry.spawn_batch(food_to_spawn);

    let entity = registry.spawn((
        Position::new(1., 1.),
        Shape::new(0.4, 0.4, (150, 150, 150, 150)),
        Hunger::new(),
        Movement::new(),
        State { state: IDLE },
    ));

    let mut entity_commands: Vec<EntityCommand> = vec![];
    let mut behaviors: HashMap<Entity, BehaviorList> = HashMap::new();
    behaviors.insert(entity, vec![behaviors::do_nothing()]);

    let mut knowledges: HashMap<Entity, Knowledge> = HashMap::new();
    knowledges.insert(
        entity,
        Knowledge {
            own_id: entity,
            target: None,
            destination_x: 0.0,
            destination_y: 0.0,
            inventory: vec![],
            map: Default::default(),
        },
    );

    // entity tasks
    let mut entity_tasks: HashMap<Entity, Vec<Box<dyn EntityTask>>> = HashMap::new();
    //

    let mut instant = Instant::now();
    let mut behavior_last_updated = Instant::now();
    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut entity_commands, &mut registry);

        // process entity commands
        while !entity_commands.is_empty() {
            // break;
            let entity_event = entity_commands.pop().unwrap();

            match entity_event.event_type {
                MoveToPosition => {
                    // dispatch MoveToBehavior for an entity
                    let mut entity_behaviors = behaviors.get_mut(&entity_event.entity).unwrap();
                    entity_behaviors.insert(0, behaviors::move_to_position());
                    // add info to knowledge
                    let mut knowledge = knowledges.get_mut(&entity_event.entity).unwrap();
                    knowledge.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    knowledge.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
                EntityCommandType::RemoveFromMap => {
                    registry.remove_one::<Position>(entity_event.entity);
                }
            }
        }

        if instant - behavior_last_updated > Duration::from_secs(10) {
            choose_behaviors(
                &mut behaviors,
                &mut knowledges,
                &mut entity_commands,
                &mut registry,
            );
            behavior_last_updated = Instant::now();
        }
        run_behaviors(
            &mut behaviors,
            &mut knowledges,
            &mut entity_commands,
            &mut registry,
        );
        movement(&mut registry);
        hunger(instant, &mut registry);

        draw(&mut window, &properties, &mut registry);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        instant = Instant::now();
    }

    Ok(())
}
