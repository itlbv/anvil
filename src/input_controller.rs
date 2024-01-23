use crate::EntityCommandType::Move;
use crate::{util, EntityCommand, Position, Properties};
use hecs::World;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::{EventPump, Sdl};

pub struct InputController {
    sdl_events: EventPump,
}

impl InputController {
    pub fn new(sdl_context: &Sdl) -> Self {
        Self {
            sdl_events: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn update(
        &mut self,
        properties: &mut Properties,
        entity_events: &mut Vec<EntityCommand>,
        world: &mut World,
    ) {
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
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => right_mouse_click(x, y, properties, entity_events),
                _ => {}
            }
        }
    }
}

fn left_mouse_click(x_screen: i32, y_screen: i32, properties: &mut Properties, world: &mut World) {
    let x_world = util::screen_to_world(x_screen, 50);
    let y_world = util::screen_to_world(y_screen, 50);

    // find close entity
    for (id, pos) in world.query_mut::<&Position>() {
        if (pos.x - x_world).abs() < 0.5 && (pos.y - y_world).abs() < 0.5 {
            properties.selected_entity = Option::from(id);
        }
    }
}

fn right_mouse_click(
    x_screen: i32,
    y_screen: i32,
    properties: &mut Properties,
    entity_events: &mut Vec<EntityCommand>,
) {
    // if entity is selected add move event to it with mouse position
    match properties.selected_entity {
        None => {
            return;
        }
        Some(entity) => {
            let x_world = util::screen_to_world(x_screen, 50);
            let y_world = util::screen_to_world(y_screen, 50);
            entity_events.push(EntityCommand {
                entity,
                event_type: Move,
                param: [
                    (String::from("x"), x_world.to_string()),
                    (String::from("y"), y_world.to_string()),
                ]
                .into(),
            })
        }
    }
}
