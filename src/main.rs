use crate::ActionType::{AddMove, RemoveMove};
use hecs::{Entity, World};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode::Blend;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use std::collections::HashMap;
use std::time::Duration;

struct Window {
    sdl_canvas: WindowCanvas,
}

impl Window {
    pub fn new(sdl_context: &Sdl) -> Self {
        let sdl_video = sdl_context.video().unwrap();
        let sdl_window = sdl_video
            .window("Anvil", 1200, 800)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut sdl_canvas = sdl_window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        sdl_canvas.set_blend_mode(Blend);

        Self { sdl_canvas }
    }

    pub fn start_frame(&mut self) {
        self.sdl_canvas.set_draw_color(Color::RGB(50, 50, 50));
        self.sdl_canvas.clear();
    }

    pub fn present_frame(&mut self) {
        self.sdl_canvas.present();
    }

    pub fn draw_rect(
        &mut self,
        x_world: f32,
        y_world: f32,
        w_world: f32,
        h_world: f32,
        color: (u8, u8, u8, u8),
    ) {
        self.sdl_canvas
            .set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));

        let x = Window::world_to_screen(x_world, 50);
        let y = Window::world_to_screen(y_world, 50);
        let w = Window::world_to_screen(w_world, 50);
        let h = Window::world_to_screen(h_world, 50);

        self.sdl_canvas
            .fill_rect(Rect::new(x, y, w as u32, h as u32))
            .expect("Error drawing rectangle.");
    }

    pub fn draw_dot(&mut self, x_world: f32, y_world: f32, color: (u8, u8, u8, u8)) {
        self.sdl_canvas
            .set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));

        let x = Window::world_to_screen(x_world, 50);
        let y = Window::world_to_screen(y_world, 50);

        self.sdl_canvas
            .draw_point(Point::new(x, y))
            .expect("Error drawing point.");
    }

    fn world_to_screen(world: f32, zoom_factor: usize) -> i32 {
        (world * zoom_factor as f32) as i32
    }
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}

struct Shape {
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8, u8),
}

struct MoveTask {
    done: bool,
    destination_x: f32,
    destination_y: f32,
}

#[derive(PartialEq)]
enum ActionType {
    AddMove,
    RemoveMove,
}

struct Action {
    entity: Entity,
    action: ActionType,
    map: HashMap<String, String>,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump()?;

    let mut world = World::new();

    let entity_1 = world.spawn((
        Position { x: 1., y: 1. },
        Shape {
            width: 0.4,
            height: 0.4,
            color: (150, 150, 150, 255),
        },
    ));

    let mut actions: Vec<Action> = vec![];
    actions.push(Action {
        entity: entity_1,
        action: AddMove,
        map: [
            ("x".to_string(), "5".to_string()),
            ("y".to_string(), "3".to_string()),
        ]
        .into(),
    });

    let mut iterations = 0;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        window.start_frame();

        for (_, (pos, shape)) in world.query_mut::<(&Position, &Shape)>() {
            window.draw_rect(
                pos.x - shape.width / 2.,
                pos.y - shape.width / 2.,
                shape.width,
                shape.height,
                shape.color,
            );
            window.draw_dot(pos.x, pos.y, (255, 255, 255, 255));
        }

        for (id, (pos, move_task)) in world.query_mut::<(&mut Position, &mut MoveTask)>() {
            // get distance to destination
            let mut dist_x = move_task.destination_x - pos.x;
            let mut dist_y = move_task.destination_y - pos.y;

            // normalise direction
            let direction_x = dist_x / dist_x.hypot(dist_y);
            let direction_y = dist_y / dist_x.hypot(dist_y);

            // modify position
            pos.x += direction_x * 0.07;
            pos.y += direction_y * 0.07;

            // signal that the movement is done
            if move_task.destination_x - pos.x < 0.2 && move_task.destination_y - pos.y < 0.2 {
                actions.push(Action {
                    entity: id,
                    action: RemoveMove,
                    map: HashMap::default(),
                });
                println!("RemoveMove added! Iteration {iterations}")
            }
        }

        while !actions.is_empty() {
            let action = actions.pop().unwrap();
            if action.action == AddMove {
                let dest_x = action.map["x"].parse::<f32>().unwrap();
                let dest_y = action.map["y"].parse::<f32>().unwrap();
                let move_task = MoveTask {
                    done: false,
                    destination_x: dest_x,
                    destination_y: dest_y,
                };
                world
                    .insert_one(action.entity, move_task)
                    .expect("Error adding Move task");
                println!("Move task added! Iteration {iterations}")
            }
            if action.action == RemoveMove {
                world
                    .remove_one::<MoveTask>(action.entity)
                    .expect("Error removing Move task");
                println!("Move task removed! Iteration {iterations}")
            }
            println!("Actions is empty. Iteration {iterations}")
        }

        window.present_frame();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        iterations += 1;
    }

    Ok(())
}
