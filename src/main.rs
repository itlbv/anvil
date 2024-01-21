use crate::EntityEventType::{StartMove, StopMove};
use hecs::{Entity, World};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode::Blend;
use sdl2::render::WindowCanvas;
use sdl2::{EventPump, Sdl};
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

        let x = world_to_screen(x_world, 50);
        let y = world_to_screen(y_world, 50);
        let w = world_to_screen(w_world, 50);
        let h = world_to_screen(h_world, 50);

        self.sdl_canvas
            .fill_rect(Rect::new(x, y, w as u32, h as u32))
            .expect("Error drawing rectangle.");
    }

    pub fn draw_dot(&mut self, x_world: f32, y_world: f32, color: (u8, u8, u8, u8)) {
        self.sdl_canvas
            .set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));

        let x_screen = world_to_screen(x_world, 50);
        let y_screen = world_to_screen(y_world, 50);

        self.sdl_canvas
            .draw_point(Point::new(x_screen, y_screen))
            .expect("Error drawing point.");
    }

    pub fn draw_selection_marker(&mut self, x_world: f32, y_world: f32) {
        self.sdl_canvas.set_draw_color(Color::RGBA(0, 200, 0, 255));

        let p_1_x_world = x_world - 0.3;
        let p_1_y_world = y_world + 0.1;

        let p_2_x_world = p_1_x_world;
        let p_2_y_world = p_1_y_world + 0.2;

        let p_3_x_world = p_2_x_world + 0.6;
        let p_3_y_world = p_2_y_world;

        let p_4_x_world = p_3_x_world;
        let p_4_y_world = p_3_y_world - 0.2;

        self.sdl_canvas
            .draw_line(
                Point::new(
                    world_to_screen(p_1_x_world, 50),
                    world_to_screen(p_1_y_world, 50),
                ),
                Point::new(
                    world_to_screen(p_2_x_world, 50),
                    world_to_screen(p_2_y_world, 50),
                ),
            )
            .expect("Error drawing line");

        self.sdl_canvas
            .draw_line(
                Point::new(
                    world_to_screen(p_2_x_world, 50),
                    world_to_screen(p_2_y_world, 50),
                ),
                Point::new(
                    world_to_screen(p_3_x_world, 50),
                    world_to_screen(p_3_y_world, 50),
                ),
            )
            .expect("Error drawing line");

        self.sdl_canvas
            .draw_line(
                Point::new(
                    world_to_screen(p_3_x_world, 50),
                    world_to_screen(p_3_y_world, 50),
                ),
                Point::new(
                    world_to_screen(p_4_x_world, 50),
                    world_to_screen(p_4_y_world, 50),
                ),
            )
            .expect("Error drawing line");
    }
}

fn world_to_screen(world: f32, zoom_factor: usize) -> i32 {
    (world * zoom_factor as f32) as i32
}

fn screen_to_world(screen: i32, zoom_factor: usize) -> f32 {
    screen as f32 / zoom_factor as f32
}

struct InputController {
    sdl_events: EventPump,
}

impl InputController {
    fn new(sdl_context: &Sdl) -> Self {
        Self {
            sdl_events: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn update(&mut self, properties: &mut Properties, world: &mut World) {
        for event in self.sdl_events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => properties.quit = true,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => left_mouse_click(x, y, properties, world),

                _ => {}
            }
        }
    }
}

fn left_mouse_click(x_screen: i32, y_screen: i32, properties: &mut Properties, world: &mut World) {
    let x_world = screen_to_world(x_screen, 50);
    let y_world = screen_to_world(y_screen, 50);

    // find close entity
    for (id, pos) in world.query_mut::<&Position>() {
        if (pos.x - x_world).abs() < 0.5 && (pos.y - y_world).abs() < 0.5 {
            properties.selected_entity = Option::from(id);
        }
    }
}

struct Properties {
    quit: bool,
    selected_entity: Option<Entity>,
}

struct Position {
    pub x: f32,
    pub y: f32,
}

struct Shape {
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8, u8),
}

struct Move {
    active: bool,
    destination_x: f32,
    destination_y: f32,
}

#[derive(PartialEq)]
enum EntityEventType {
    StartMove,
    StopMove,
}

struct EntityEvent {
    entity: Entity,
    event_type: EntityEventType,
    param: HashMap<String, String>,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut input_controller = InputController::new(&sdl_context);

    let mut world = World::new();
    let mut properties = Properties {
        quit: false,
        selected_entity: None,
    };

    let entity_1 = world.spawn((
        Move {
            active: false,
            destination_x: 0.0,
            destination_y: 0.0,
        },
        Position { x: 1., y: 1. },
        Shape {
            width: 0.4,
            height: 0.4,
            color: (150, 150, 150, 255),
        },
    ));

    let mut entity_events: Vec<EntityEvent> = vec![];
    // entity_events.push(EntityEvent {
    //     entity: entity_1,
    //     event_type: StartMove,
    //     param: [
    //         (String::from("x"), String::from("5")),
    //         (String::from("y"), String::from("3")),
    //     ]
    //     .into(),
    // });

    let mut iterations = 0;

    'main: loop {
        if properties.quit {
            break 'main;
        }

        // input
        input_controller.update(&mut properties, &mut world);

        // entity_events
        while !entity_events.is_empty() {
            let entity_event = entity_events.pop().unwrap();

            match entity_event.event_type {
                StartMove => {
                    let mut move_task = world
                        .get::<&mut Move>(entity_event.entity)
                        .expect("Error getting Move component");
                    move_task.active = true;
                    move_task.destination_x = entity_event.param["x"].parse::<f32>().unwrap();
                    move_task.destination_y = entity_event.param["y"].parse::<f32>().unwrap();
                }
                StopMove => {}
            }
        }

        // move
        for (id, (pos, move_task)) in world.query_mut::<(&mut Position, &mut Move)>() {
            if !move_task.active {
                continue;
            }

            // get distance to destination
            let mut dist_x = move_task.destination_x - pos.x;
            let mut dist_y = move_task.destination_y - pos.y;

            // normalise direction
            let direction_x = dist_x / dist_x.hypot(dist_y);
            let direction_y = dist_y / dist_x.hypot(dist_y);

            // modify position
            pos.x += direction_x * 0.07;
            pos.y += direction_y * 0.07;

            // movement is done
            if move_task.destination_x - pos.x < 0.05 && move_task.destination_y - pos.y < 0.05 {
                pos.x = move_task.destination_x;
                pos.y = move_task.destination_y;
                move_task.active = false;
                println!("Movement is done")
            }
        }

        window.start_frame();

        // draw entities
        for (id, (pos, shape)) in world.query_mut::<(&Position, &Shape)>() {
            window.draw_rect(
                pos.x - shape.width / 2.,
                pos.y - shape.width / 2.,
                shape.width,
                shape.height,
                shape.color,
            );
            window.draw_dot(pos.x, pos.y, (255, 255, 255, 255));

            // draw selection marker if entity is selected
            match properties.selected_entity {
                None => {}
                Some(selected_entity) => {
                    if selected_entity == id {
                        window.draw_selection_marker(pos.x, pos.y);
                    }
                }
            }
        }

        window.present_frame();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        iterations += 1;
    }

    Ok(())
}
