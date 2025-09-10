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
use std::path::PathBuf;
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

#[derive(Debug, Clone)]
enum Mode {
    Normal,
    Record(PathBuf),
    Replay(PathBuf),
}

#[derive(Debug, Clone)]
struct Cli {
    mode: Mode,
    ticks: Option<u64>,
    seed: Option<u64>,   // optional; use if you want
    sim_hz: Option<u32>, // optional; use if you want
}

fn usage() -> &'static str {
    "Usage:
      anvil [--record FILE | --replay FILE] [--ticks N] [--seed U64] [--sim-hz HZ]

    Examples:
      anvil --record run.bin --ticks 1200
      anvil --replay=run.bin

    Notes:
      --record and --replay are mutually exclusive.
      --ticks stops the sim after N fixed ticks and prints:
        FINAL end_tick=<N> world_hash=<0x...>
"
}

fn parse_args() -> Result<Cli, String> {
    let mut mode = Mode::Normal;
    let mut ticks: Option<u64> = None;
    let mut seed: Option<u64> = None;
    let mut sim_hz: Option<u32> = None;

    let mut it = std::env::args().skip(1).peekable();
    while let Some(arg) = it.next() {
        // Help
        if arg == "-h" || arg == "--help" {
            return Err(usage().to_string());
        }

        // --foo=bar style
        if let Some(val) = arg.strip_prefix("--record=") {
            if !matches!(mode, Mode::Normal) {
                return Err("Cannot combine --record and --replay".into());
            }
            mode = Mode::Record(PathBuf::from(val));
            continue;
        }
        if let Some(val) = arg.strip_prefix("--replay=") {
            if !matches!(mode, Mode::Normal) {
                return Err("Cannot combine --record and --replay".into());
            }
            mode = Mode::Replay(PathBuf::from(val));
            continue;
        }
        if let Some(val) = arg.strip_prefix("--ticks=") {
            ticks = Some(
                val.parse()
                    .map_err(|_| "Invalid --ticks value; expected u64".to_string())?,
            );
            continue;
        }
        if let Some(val) = arg.strip_prefix("--seed=") {
            // allow hex: 0x...
            let v = if let Some(hex) = val.strip_prefix("0x") {
                u64::from_str_radix(hex, 16)
            } else {
                val.parse()
            };
            seed = Some(
                v.map_err(|_| "Invalid --seed value; expected u64 (allow 0x...)".to_string())?,
            );
            continue;
        }
        if let Some(val) = arg.strip_prefix("--sim-hz=") {
            sim_hz = Some(
                val.parse()
                    .map_err(|_| "Invalid --sim-hz value; expected u32".to_string())?,
            );
            continue;
        }

        // Space-separated variants: --flag VAL
        match arg.as_str() {
            "--record" => {
                let p = it
                    .next()
                    .ok_or("--record requires a file path".to_string())?;
                if !matches!(mode, Mode::Normal) {
                    return Err("Cannot combine --record and --replay".into());
                }
                mode = Mode::Record(PathBuf::from(p));
            }
            "--replay" => {
                let p = it
                    .next()
                    .ok_or("--replay requires a file path".to_string())?;
                if !matches!(mode, Mode::Normal) {
                    return Err("Cannot combine --record and --replay".into());
                }
                mode = Mode::Replay(PathBuf::from(p));
            }
            "--ticks" => {
                let v = it.next().ok_or("--ticks requires a number".to_string())?;
                ticks = Some(
                    v.parse()
                        .map_err(|_| "Invalid --ticks value; expected u64".to_string())?,
                );
            }
            "--seed" => {
                let v = it.next().ok_or("--seed requires a number".to_string())?;
                let parsed = if let Some(hex) = v.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16)
                } else {
                    v.parse()
                };
                seed =
                    Some(parsed.map_err(|_| {
                        "Invalid --seed value; expected u64 (allow 0x...)".to_string()
                    })?);
            }
            "--sim-hz" => {
                let v = it.next().ok_or("--sim-hz requires a number".to_string())?;
                sim_hz = Some(
                    v.parse()
                        .map_err(|_| "Invalid --sim-hz value; expected u32".to_string())?,
                );
            }
            other => {
                return Err(format!("Unknown option: {other}\n{usage}", usage = usage()));
            }
        }
    }

    Ok(Cli {
        mode,
        ticks,
        seed,
        sim_hz,
    })
}

fn main() -> Result<(), String> {
    // Parse CLI
    let cli = parse_args().map_err(|e| e.to_string())?;

    // Defaults
    let mut sim_hz: u32 = 60;
    if let Some(hz) = cli.sim_hz {
        sim_hz = hz;
    }
    let mut sim = sim_loop::SimLoop::new(sim_hz);

    let mut run_seed = 0xDEADBEEFCAFEBABEu64;
    if let Some(seed) = cli.seed {
        run_seed = seed;
    }
    let run = RngRun::new(run_seed);

    // Mode
    let mut recorder: Option<Recorder> = None;
    let mut player: Option<Player> = None;
    match &cli.mode {
        Mode::Record(path) => {
            let meta = RunMeta {
                sim_hz: sim_hz as u32,
                seed: run_seed,
                version: 1,
            };
            recorder = Some(Recorder::new(&path, meta).map_err(|e| e.to_string())?);
        }
        Mode::Replay(path) => {
            let p = Player::new(&path).map_err(|e| e.to_string())?;
            if p.meta.sim_hz != sim_hz as u32 {
                sim = sim_loop::SimLoop::new(p.meta.sim_hz);
            }
            // If you want to force the recorded seed:
            // run = RngRun::new(p.meta.seed);
            player = Some(p);
        }
        Mode::Normal => {}
    }

    // SDL2 rendering and input init
    let sdl_context = sdl2::init()?;
    let mut window = Window::new(&sdl_context);
    let mut input_controller = InputController::new(&sdl_context);

    let mut registry = ComponentRegistry::new();
    let map = Map::new(24, 16);

    // Entities spawn
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

    let mut properties = Properties {
        quit: false,
        selected_entity: None,
        draw_map_grid: true,
    };

    let cli_ticks_limit = cli.ticks;

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

            // Exiting if ticks limit from CLI args reached
            if let Some(limit) = cli_ticks_limit {
                if sim.tick.0 >= limit {
                    properties.quit = true;
                }
            }

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
