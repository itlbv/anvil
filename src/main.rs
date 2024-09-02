mod behaviors;
mod btree;
mod components;
mod entity_commands;
mod input_controller;
mod map;
mod systems;
mod util;
mod window;

use crate::btree::BehaviorTreeNode;
use crate::components::StateType::{IDLE, MOVE};
use crate::components::{Food, Hunger, Movement, Position, Shape, State, StateType};
use crate::entity_commands::process_entity_commands;
use crate::entity_commands::EntityCommand;
use crate::input_controller::InputController;
use crate::systems::{choose_behaviors, hunger, movement, render_frame, run_behaviors};
use crate::window::Window;
use hecs::Entity;
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

use crate::map::Map;
use hecs::World as ComponentRegistry;

type BehaviorList = Vec<Box<dyn BehaviorTreeNode>>;

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
    draw_map_grid: bool,
}

struct Knowledge {
    own_id: Entity,
    target: Option<Entity>,
    destination_x: f32,
    destination_y: f32,
    inventory: Vec<Entity>,
    map: HashMap<String, String>,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut input_controller = InputController::new(&sdl_context);

    let mut properties = Properties {
        quit: false,
        selected_entity: None,
        draw_map_grid: true,
    };

    let mut map = Map::new(10, 10);

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

    let mut instant = Instant::now();
    let mut behavior_last_updated = Instant::now();
    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut entity_commands, &mut registry);

        process_entity_commands(
            &mut entity_commands,
            &mut knowledges,
            &mut behaviors,
            &mut registry,
        );

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

        render_frame(&mut window, &properties, &map, &mut registry);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        instant = Instant::now();
    }

    Ok(())
}
