use amethyst::{
    assets::{PrefabLoader, RonFormat},
    controls::{HideCursor},
    ecs::WorldExt,
    input::{is_key_down, is_mouse_button_down},
    prelude::*,
    renderer::{
        rendy::mesh::{Normal, Position, TexCoord},
    },
    utils::{scene::BasicScenePrefab},
    winit::{MouseButton, VirtualKeyCode},
};

pub type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

pub struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("fly_camera.ron", RonFormat, ())
        });
        data.world
            .create_entity()
            .named("Fly Camera Scene")
            .with(prefab_handle)
            .build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = true;
            }
        }
        Trans::None
    }
}