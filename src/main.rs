use hecs::World;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
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
}

struct Position {
    pub x: usize,
    pub y: usize,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let mut window = Window::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump()?;

    let mut world = World::new();

    let entity_1 = world.spawn((Position { x: 1, y: 1 },));
    let entity_2 = world.spawn((Position { x: 3, y: 3 },));

    for (id, &ref pos) in world.query_mut::<&Position>() {
        println!("Position: x: {}, y: {}", pos.x, pos.y)
    }

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
        window.present_frame();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
