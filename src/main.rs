mod behaviors;
mod btree;
mod components;
mod entity_commands;
mod input_controller;
mod map;
mod recipes;
mod rng;
mod sim_loop;
mod systems;
mod time;
mod util;
mod window;
mod world_hash;

use crate::btree::BehaviorTreeNode;
use crate::components::StateType::{Idle, Move};
use crate::components::{Food, Hunger, Movement, Position, Shape, State, StateType, Stone, Wood};
use crate::entity_commands::process_entity_commands;
use crate::entity_commands::EntityCommand;
use crate::input_controller::InputController;
use crate::recipes::Recipe;
use crate::rng::rng_for_tick;
use crate::rng::RngRun;
use crate::systems::{choose_behaviors, hunger, movement, render_frame, run_behaviors};
use crate::window::Window;
use hecs::Entity;
use rand::Rng;
use std::any::TypeId;
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

struct EntityWithType {
    type_id: TypeId,
    entity: Entity,
}

impl EntityWithType {
    pub fn new(type_id: TypeId, entity: Entity) -> Self {
        Self { type_id, entity }
    }
}

struct Knowledge {
    own_id: Entity,
    target: Option<EntityWithType>,
    destination_x: f32,
    destination_y: f32,
    recipe: Option<Recipe>,
    inventory: HashMap<TypeId, Vec<Entity>>,
    param: HashMap<String, String>,
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

    let map = Map::new(24, 16);

    let mut registry = ComponentRegistry::new();

    let mut sim = sim_loop::SimLoop::new(60);
    let run_seed = 0xDEADBEEFCAFEBABEu64; // replace with CLI arg/env for reproducibility
    let run = RngRun::new(run_seed);

    let mut rand = rng_for_tick(&run, 0, 42); // tick=0, stream=42 for "spawn"
    let food_to_spawn = (0..6).map(|_| {
        let pos = Position::new(
            rand.random_range(2..10) as f32 + 0.5,
            rand.random_range(2..10) as f32 + 0.5,
        );
        let shape = Shape::new(0.2, 0.2, (150, 40, 40, 255));
        let food = Food {
            type_id: TypeId::of::<Food>(),
        };
        (pos, shape, food)
    });
    registry.spawn_batch(food_to_spawn);

    let wood_to_spawn = (0..3).map(|_| {
        let pos = Position::new(
            rand.random_range(2..10) as f32 + 0.5,
            rand.random_range(2..10) as f32 + 0.5,
        );
        let shape = Shape::new(0.2, 0.2, (170, 70, 0, 255));
        (
            pos,
            shape,
            Wood {
                type_id: TypeId::of::<Wood>(),
            },
        )
    });
    registry.spawn_batch(wood_to_spawn);

    let stone_to_spawn = (0..3).map(|_| {
        let pos = Position::new(
            rand.random_range(2..10) as f32 + 0.5,
            rand.random_range(2..10) as f32 + 0.5,
        );
        let shape = Shape::new(0.2, 0.2, (170, 170, 170, 255));
        (
            pos,
            shape,
            Stone {
                type_id: TypeId::of::<Stone>(),
            },
        )
    });
    registry.spawn_batch(stone_to_spawn);

    let entity = registry.spawn((
        Position::new(1.5, 1.5),
        Shape::new(0.4, 0.4, (150, 150, 150, 150)),
        Hunger::new(),
        Movement::new(),
        State { state: Idle },
    ));

    let mut entity_commands: Vec<EntityCommand> = vec![];
    let mut behaviors: HashMap<Entity, BehaviorList> = HashMap::new();
    // behaviors.insert(entity, vec![behaviors::do_nothing()]);
    behaviors.insert(entity, vec![behaviors::build_house()]);

    let mut knowledges: HashMap<Entity, Knowledge> = HashMap::new();
    knowledges.insert(
        entity,
        Knowledge {
            own_id: entity,
            target: None,
            destination_x: 0.0,
            destination_y: 0.0,
            recipe: Option::None,
            inventory: HashMap::new(),
            param: Default::default(),
        },
    );

    let start_instant = Instant::now();
    let sim_elapsed = Duration::ZERO;
    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut entity_commands, &mut registry);

        let steps = sim.begin_frame();
        for _ in 0..steps {
            let tick = sim.tick.0;

            // deterministic RNG for this tick & domain "ai" (stream id = 1)
            let rng_ai = rng_for_tick(&run, tick, 1);

            // --- your per-tick simulation systems ---
            process_entity_commands(
                &mut entity_commands,
                &mut knowledges,
                &mut behaviors,
                &mut registry,
            );

            run_behaviors(
                &mut behaviors,
                &mut knowledges,
                &mut entity_commands,
                &mut registry,
            );

            movement(&mut registry);
            hunger(sim.fixed.seconds, &mut registry);

            // advance deterministic tick counter
            sim.advance_tick();
        }

        render_frame(&mut window, &properties, &map, &mut registry);
    }

    Ok(())
}
