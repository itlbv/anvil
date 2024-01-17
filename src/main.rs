use hecs::World;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode::Blend;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
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
            .expect("Can't draw rectangle!");
    }

    fn world_to_screen(world: f32, zoom_factor: usize) -> i32 {
        (world * zoom_factor as f32) as i32
    }
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

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump()?;

    let mut world = World::new();

    world.spawn((
        Position { x: 1., y: 1. },
        Shape {
            width: 0.4,
            height: 0.4,
            color: (100, 100, 100, 255),
        },
    ));
    world.spawn((
        Position { x: 3., y: 3. },
        Shape {
            width: 0.4,
            height: 0.4,
            color: (100, 100, 100, 255),
        },
    ));

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

        for (id, (pos, shape)) in world.query_mut::<(&Position, &Shape)>() {
            window.draw_rect(pos.x, pos.y, shape.width, shape.height, shape.color);
        }

        window.present_frame();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
