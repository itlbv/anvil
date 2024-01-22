pub fn world_to_screen(world: f32, zoom: usize) -> i32 {
    (world * zoom as f32) as i32
}

pub fn screen_to_world(screen: i32, zoom: usize) -> f32 {
    screen as f32 / zoom as f32
}
