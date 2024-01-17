use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::BlendMode::Blend;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let sdl_video = sdl_context.video()?;
    let sdl_window = sdl_video
        .window("Anvil", 1200, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut sdl_canvas = sdl_window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    sdl_canvas.set_blend_mode(Blend);

    sdl_canvas.set_draw_color(Color::RGB(50, 50, 50));
    sdl_canvas.clear();
    sdl_canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

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

        sdl_canvas.clear();
        sdl_canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
