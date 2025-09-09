mod behaviors;
mod btree;
mod components;
mod entity_commands;
mod entity_serde;
mod input_controller;
mod map;
mod recipes;
mod rng;
mod sim_loop;
mod systems;
mod time;
mod trace;
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
use trace::{Player, PropsDelta, Recorder, RunMeta, TickEvents};

use crate::map::Map;
use hecs::World as ComponentRegistry;

type BehaviorList = Vec<Box<dyn BehaviorTreeNode>>;

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
    draw_map_grid: bool,
}

fn props_delta(before: &Properties, after: &Properties) -> Option<PropsDelta> {
    let mut d = PropsDelta {
        selected_entity: None,
        draw_map_grid: None,
        quit: None,
    };
    if before.selected_entity != after.selected_entity {
        d.selected_entity = after.selected_entity;
    }
    if before.draw_map_grid != after.draw_map_grid {
        d.draw_map_grid = Some(after.draw_map_grid);
    }
    if before.quit != after.quit {
        d.quit = Some(after.quit);
    }

    if d.selected_entity.is_some() || d.draw_map_grid.is_some() || d.quit.is_some() {
        Some(d)
    } else {
        None
    }
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

enum Mode {
    Normal,
    Record(String),
    Replay(String),
}

fn detect_mode() -> Mode {
    let mut rec = None;
    let mut rep = None;
    for arg in std::env::args().skip(1) {
        if let Some(p) = arg.strip_prefix("--record=") {
            rec = Some(p.to_string());
        }
        if let Some(p) = arg.strip_prefix("--replay=") {
            rep = Some(p.to_string());
        }
    }
    if let Some(p) = rec {
        Mode::Record(p)
    } else if let Some(p) = rep {
        Mode::Replay(p)
    } else {
        Mode::Normal
    }
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

    let mode = detect_mode();
    let mut recorder: Option<Recorder> = None;
    let mut player: Option<Player> = None;

    match &mode {
        Mode::Record(path) => {
            // align meta with your current settings
            let meta = RunMeta {
                sim_hz: 60,
                seed: run_seed,
                version: 1,
            };
            recorder = Some(Recorder::new(path, meta).map_err(|e| e.to_string())?);
        }
        Mode::Replay(path) => {
            let p = Player::new(path).map_err(|e| e.to_string())?;
            // optional: align to recorded meta
            if p.meta.sim_hz != 60 {
                sim = sim_loop::SimLoop::new(p.meta.sim_hz);
            }
            // If you want to force the seed from the file:
            // run = RngRun::new(p.meta.seed);
            player = Some(p);
        }
        Mode::Normal => {}
    }

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
        match &mut player {
            // --- REPLAY: inject recorded events for this tick ---
            Some(p) => {
                let evs = p.next_for_tick(sim.tick.0).map_err(|e| e.to_string())?;
                for ev in evs {
                    if let Some(pd) = ev.props {
                        if let Some(sel) = pd.selected_entity {
                            properties.selected_entity = Some(sel);
                        }
                        if let Some(b) = pd.draw_map_grid {
                            properties.draw_map_grid = b;
                        }
                        if let Some(b) = pd.quit {
                            properties.quit = b;
                        }
                    }
                    entity_commands.extend(ev.commands.into_iter());
                }
            }
            // --- NORMAL / RECORD: poll SDL, then optionally record the delta ---
            None => {
                let before = Properties {
                    quit: properties.quit,
                    selected_entity: properties.selected_entity,
                    draw_map_grid: properties.draw_map_grid,
                };
                let pre_len = entity_commands.len();

                input_controller.update(&mut properties, &mut entity_commands, &mut registry);

                if let Some(rec) = &mut recorder {
                    let after = Properties {
                        quit: properties.quit,
                        selected_entity: properties.selected_entity,
                        draw_map_grid: properties.draw_map_grid,
                    };
                    let pd = props_delta(&before, &after);
                    let cmds_tail = entity_commands[pre_len..].to_vec(); // EntityCommand: Clone + Serialize
                    let ev = TickEvents {
                        tick: sim.tick.0,
                        props: pd,
                        commands: cmds_tail,
                    };
                    rec.push(&ev).map_err(|e| e.to_string())?;
                }
            }
        }

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

            // Auto-exit when we reach the recorded end in replay mode
            if let Some(p) = &player {
                if sim.tick.0 >= p.trailer.end_tick {
                    properties.quit = true;
                }
            }

            if sim.tick.0 % 600 == 0 {
                let hash = world_hash::world_hash(&registry);
                println!("after tick {}, world_hash={:#018x}", sim.tick.0, hash);
            }
        }

        render_frame(&mut window, &properties, &map, &mut registry);
    }

    Ok(())
}
