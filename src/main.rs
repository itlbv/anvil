mod behaviors;
mod btree;
mod command_bus;
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
use crate::components::StateType::Idle;
use crate::components::{Food, Hunger, Movement, Position, Shape, State, Stone, Wood};
use crate::entity_commands::EntityCommand;
use crate::entity_commands::{process_commands, resolve_commands};
use crate::input_controller::InputController;
use crate::map::Map;
use crate::recipes::Recipe;
use crate::rng::{rng_for_tick, RngRun};
use crate::systems::{hunger, movement, render_frame, run_behaviors};
use crate::window::Window;
use hecs::Entity;
use hecs::World as ComponentRegistry;
use rand::Rng;
use std::any::TypeId;
use std::collections::HashMap;
use trace::{Player, PropsDelta, Recorder, RunMeta, TickEvents, Trailer};
use world_hash as wh;

type BehaviorList = Vec<Box<dyn BehaviorTreeNode>>;

#[derive(Copy, Clone)]
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
    use std::iter::Peekable;
    let mut args: Peekable<_> = std::env::args().skip(1).peekable();
    let mut rec: Option<String> = None;
    let mut rep: Option<String> = None;

    while let Some(arg) = args.next() {
        if let Some(p) = arg.strip_prefix("--record=") {
            rec = Some(p.to_string());
            continue;
        }
        if let Some(p) = arg.strip_prefix("--replay=") {
            rep = Some(p.to_string());
            continue;
        }
        if arg == "--record" {
            if let Some(p) = args.next() {
                rec = Some(p);
            }
            continue;
        }
        if arg == "--replay" {
            if let Some(p) = args.next() {
                rep = Some(p);
            }
            continue;
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

    // ------------------------
    // Mode + meta BEFORE spawns
    // ------------------------
    let mode = detect_mode();
    let mut recorder: Option<Recorder> = None;
    let mut player: Option<Player> = None;

    // If replay, lock Hz+seed from the file before building sim/run & spawning
    let (sim_hz, run_seed) = match &mode {
        Mode::Replay(path) => {
            let p = Player::new(path).map_err(|e| e.to_string())?;
            let hz = p.meta.sim_hz;
            let seed = p.meta.seed;
            player = Some(p);
            (hz, seed)
        }
        _ => (60, 0xDEADBEEFCAFEBABEu64),
    };

    let mut sim = sim_loop::SimLoop::new(sim_hz);
    let run = RngRun::new(run_seed);

    if let Mode::Record(path) = &mode {
        let meta = RunMeta {
            sim_hz,
            seed: run_seed,
            version: 1,
        };
        recorder = Some(Recorder::new(path, meta).map_err(|e| e.to_string())?);
    }

    // ------------------------
    // Deterministic spawns
    // ------------------------
    let mut rand = rng_for_tick(&run, 0, 42); // stream=42 "spawn"

    let food_to_spawn = (0..6).map(|_| {
        let pos = Position::new(
            rand.gen_range(2..10) as f32 + 0.5,
            rand.gen_range(2..10) as f32 + 0.5,
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
            rand.gen_range(2..10) as f32 + 0.5,
            rand.gen_range(2..10) as f32 + 0.5,
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
            rand.gen_range(2..10) as f32 + 0.5,
            rand.gen_range(2..10) as f32 + 0.5,
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

    let mut command_bus = command_bus::CommandBus::new();
    let mut behaviors: HashMap<Entity, BehaviorList> = HashMap::new();
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

    'main: loop {
        if properties.quit {
            break 'main;
        }

        // ---- Pump SDL every frame so the window stays responsive
        let before_props = properties;
        let pre_len = command_bus.incoming.len();
        input_controller.update(&mut properties, &mut command_bus.incoming, &mut registry);

        // In replay, discard any live input changes to keep determinism.
        if player.is_some() {
            command_bus.incoming.truncate(pre_len);
            properties = before_props;
        }

        // ---- Normal/Record: after polling, write deltas for THIS FRAME (we still inject per TICK below)
        if player.is_none() {
            if let Some(rec) = &mut recorder {
                let after = properties;
                let pd = props_delta(&before_props, &after);
                let cmds_tail = command_bus.incoming[pre_len..].to_vec(); // EntityCommand: Clone + Serialize
                let ev = TickEvents {
                    tick: sim.tick.0,
                    props: pd,
                    commands: cmds_tail,
                };
                rec.push(&ev).map_err(|e| e.to_string())?;
            }
        }

        // ---- Fixed-step simulation
        let steps = sim.begin_frame();
        for _ in 0..steps {
            let tick = sim.tick.0;

            // REPLAY: inject events for THIS TICK (authoritative for sim)
            if let Some(p) = &mut player {
                for ev in p.next_for_tick(tick).map_err(|e| e.to_string())? {
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
                    command_bus.incoming.extend(ev.commands.into_iter());
                }
            }

            // --- per-tick systems ---
            command_bus.begin_tick();
            resolve_commands(&mut command_bus.processing);
            process_commands(
                &mut command_bus.processing,
                &mut knowledges,
                &mut behaviors,
                &mut registry,
            );
            run_behaviors(
                &mut behaviors,
                &mut knowledges,
                &mut command_bus.incoming,
                &mut registry,
            );
            movement(&mut registry);
            hunger(sim.fixed.seconds, &mut registry);

            // advance deterministic tick counter
            sim.advance_tick();

            // Auto-exit one tick after EOF in replay
            if let Some(p) = &player {
                if p.eof_reached && sim.tick.0 > p.last_tick_seen {
                    properties.quit = true;
                }
            }

            #[cfg(feature = "hash_debug")]
            if sim.tick.0 % 600 == 0 {
                let b = world_hash::world_hash_breakdown(&registry);
                println!(
                    "tick={} total={:#018x} pos={:#018x} hun={:#018x} sta={:#018x} food={:#018x} wood={:#018x} stone={:#018x} shape={:#018x}",
                    sim.tick.0, b.total, b.pos, b.hun, b.sta, b.food, b.wood, b.stone, b.shape
                );
            }
        }

        // ---- Render once per frame
        render_frame(&mut window, &properties, &map, &mut registry);
    }

    // ---- On exit: if recording, write trailer for future strict checks
    if let Some(rec) = recorder {
        let final_hash = wh::world_hash(&registry);
        let tr = Trailer {
            end_tick: sim.tick.0,
            final_world_hash: final_hash,
        };
        rec.finish(&tr).map_err(|e| e.to_string())?;
        println!(
            "Recorded trailer: end_tick={}, hash={:#018x}",
            sim.tick.0, final_hash
        );
    }

    Ok(())
}
