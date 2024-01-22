use crate::world_to_screen;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode::Blend;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

pub struct Window {
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
