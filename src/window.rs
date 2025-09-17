use crate::util;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode::Blend;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

pub enum PaletteEntry {
    Rect {
        w: i32,
        h: i32,
        color: (u8, u8, u8, u8),
    },
}

fn init_palette_entries() -> Vec<PaletteEntry> {
    vec![
        PaletteEntry::Rect {
            w: 1,
            h: 1,
            color: (0, 0, 0, 0),
        }, // 0 - default gray
    ]
}

struct ShapePalette {
    entries: Vec<PaletteEntry>,
}

impl ShapePalette {
    fn new() -> Self {
        Self {
            entries: init_palette_entries(),
        }
    }
}

pub struct Window {
    sdl_canvas: WindowCanvas,
    camera_pos: (f32, f32),
    camera_zoom: usize,
    camera_dirty: bool,
    shape_palette: ShapePalette,
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

        Self {
            sdl_canvas,
            camera_pos: (0., 0.),
            camera_zoom: 50,
            camera_dirty: true,
            shape_palette: ShapePalette::new(),
        }
    }

    pub fn set_camera(&mut self, pos: (f32, f32), zoom: usize) {
        if self.camera_pos != pos || self.camera_zoom != zoom {
            self.camera_pos = pos;
            self.camera_zoom = zoom;
            self.camera_dirty = true;
        }
    }

    pub fn start_frame(&mut self) {
        self.sdl_canvas.set_draw_color(Color::RGB(50, 50, 50));
        self.sdl_canvas.clear();
    }

    pub fn present_frame(&mut self) {
        self.sdl_canvas.present();
    }

    pub fn draw_map_tile(&mut self, tile_pos: (u32, u32), shape_id: u16) {
        // let shape = self.shape_palette.entries[shape_id];
        // self.draw_rect(tile_pos.0 as f32, tile_pos.1 as f32, shape, 1., self.shape_palette.entries[shape_id])
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

        let x = util::world_to_screen(x_world, 50);
        let y = util::world_to_screen(y_world, 50);
        let w = util::world_to_screen(w_world, 50);
        let h = util::world_to_screen(h_world, 50);

        self.sdl_canvas
            .fill_rect(Rect::new(x, y, w as u32, h as u32))
            .expect("Error drawing rectangle with SDL Canvas.");
    }

    pub fn draw_line(&mut self, start: (f32, f32), end: (f32, f32), color: (u8, u8, u8, u8)) {
        self.sdl_canvas
            .set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));

        let start_x = util::world_to_screen(start.0, 50);
        let start_y = util::world_to_screen(start.1, 50);
        let end_x = util::world_to_screen(end.0, 50);
        let end_y = util::world_to_screen(end.1, 50);

        self.sdl_canvas
            .draw_line(Point::new(start_x, start_y), Point::new(end_x, end_y))
            .expect("error drawing line with SDL Canvas");
    }

    pub fn draw_dot(&mut self, x_world: f32, y_world: f32, color: (u8, u8, u8, u8)) {
        self.sdl_canvas
            .set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));

        let x_screen = util::world_to_screen(x_world, 50);
        let y_screen = util::world_to_screen(y_world, 50);

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
                    util::world_to_screen(p_1_x_world, 50),
                    util::world_to_screen(p_1_y_world, 50),
                ),
                Point::new(
                    util::world_to_screen(p_2_x_world, 50),
                    util::world_to_screen(p_2_y_world, 50),
                ),
            )
            .expect("Error drawing line");

        self.sdl_canvas
            .draw_line(
                Point::new(
                    util::world_to_screen(p_2_x_world, 50),
                    util::world_to_screen(p_2_y_world, 50),
                ),
                Point::new(
                    util::world_to_screen(p_3_x_world, 50),
                    util::world_to_screen(p_3_y_world, 50),
                ),
            )
            .expect("Error drawing line");

        self.sdl_canvas
            .draw_line(
                Point::new(
                    util::world_to_screen(p_3_x_world, 50),
                    util::world_to_screen(p_3_y_world, 50),
                ),
                Point::new(
                    util::world_to_screen(p_4_x_world, 50),
                    util::world_to_screen(p_4_y_world, 50),
                ),
            )
            .expect("Error drawing line");
    }
}
