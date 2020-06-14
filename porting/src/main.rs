extern crate amethyst;
extern crate amethyst_rhusics;
extern crate rhusics_core;
extern crate rhusics_ecs;
extern crate collision;
extern crate shred;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{Loader,AssetStorage,Handle};
use amethyst::config::Config;
use amethyst::controls::{FlyControlTag,FlyControlBundle};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::core::{Time,Parent,ECSBundle};
use amethyst::ecs::{World,VecStorage,Component,Fetch,Entity,System,Join,ReadStorage,FetchMut,Entities,WriteStorage};
use amethyst::input::{InputBundle,InputHandler};
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, ElementState, Event,
                         KeyboardInput, Material, MaterialDefaults, MeshHandle, ObjFormat,
                         Pipeline, PosNormTex, Projection, RenderBundle, Rgba, Stage,
                         VirtualKeyCode, WindowEvent,Texture,MouseButton,ScreenDimensions,WindowMessages};
use amethyst::renderer::mouse::{release_cursor,grab_cursor,set_mouse_cursor_none,set_mouse_cursor};
use amethyst::shrev::EventChannel;
use amethyst::ui::{Anchor, Anchored, DrawUi, FontAsset, MouseReactive, Stretch, Stretched,
                   TtfFormat, UiBundle, UiEvent, UiFocused, UiImage, UiText,
                   UiTransform,TextEditing};
use amethyst::winit::MouseCursor;

//use amethyst_rhusics::{time_sync, DefaultBasicPhysicsBundle3,SpatialPhysicsBundle3};
use collision::{Aabb3,Ray3};
use collision::dbvt::query_ray_closest;
use collision::primitive::{Primitive3,Cuboid};
/*use rhusics_core::{CollisionShape, RigidBody,Collider,ContactEvent,Velocity,ForceAccumulator};
use rhusics_ecs::WithRigidBody;
use rhusics_ecs::physics3d::{register_physics,BodyPose3, CollisionMode,
                             CollisionStrategy, Mass3,DynamicBoundingVolumeTree3,SpatialSortingSystem3,ContactEvent3,
                             SpatialCollisionSystem3,GJK3,CurrentFrameUpdateSystem3,NextFrameSetupSystem3,ContactResolutionSystem3,Velocity3};*/
use amethyst::core::cgmath::{Deg, Array, Basis3,Basis2, One, Point3, Quaternion, Vector3,Matrix3,Zero,EuclideanSpace,Rotation};

use shred::{Dispatcher,DispatcherBuilder};

mod player;
use player::{Tool,Backpack,BlockDefinition,BlockDefinitions,BlockInstance,Inventory,UiUpdaterSystem,MineProgress};
mod ui;
use ui::{create_game_ui,load_tool_icon,UiShit,load_ui_shit,create_buy_ui};

/*
TODO

Raycast
Click mine
UI
-Layouting
-Macro for btn, auto layout by pos




*/




struct MiningSystem{
    was_down: bool,
}

impl MiningSystem{
    pub fn new() -> Self{
        MiningSystem{
            was_down: false,
        }
    }
}

impl<'a> System<'a> for MiningSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, DynamicBoundingVolumeTree3<f32>>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, BlockInstance>,
        Fetch<'a, InputHandler<String,String>>,
        FetchMut<'a, Inventory>,
        FetchMut<'a, MineProgress>,
        Fetch<'a, Time>,
        WriteStorage<'a, ForceAccumulator<Vector3<f32>, Vector3<f32>>>,
    );

    fn run(&mut self, (entities,tree,camera,transform,block_definitions,input, mut inventory, mut progress, time,mut force): Self::SystemData) {
        let btn_down = input.mouse_button_is_down(MouseButton::Left);

        if btn_down {
            for (tr, _) in (&transform, &camera).join() {
                let forward = Quaternion::from(tr.rotation).conjugate().invert() * -Vector3::unit_z();
                let ray = Ray3::new(Point3::from_vec(tr.translation), forward);
                if let Some((v, p)) = query_ray_closest(&*tree, ray) {
                    println!("hit bounding volume of {:?} at point {:?}", v.value, p);


                    /*if let Some(mut f) = force.get_mut(v.value){
                        f.add_force(Vector3::new(1.0,-10.0,1.0));
                        println!("ADDING FORCE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                    }*/


                    // TODO raycast lookat + dist check, are we mining same block that we were
                    if Some(v.value) != progress.block {
                        progress.block = Some(v.value);
                        progress.start = time.absolute_time_seconds();
                        progress.progress = 0.0;
                    }

                    progress.progress = (time.absolute_time_seconds() - progress.start) as f32 / inventory.tool.use_time;

                    if progress.progress > 1.0 {
                        progress.progress = 1.0;
                    }

                    if progress.progress == 1.0 {
                        progress.reset();

                        entities.delete(v.value);

                        // switch to block def
                        inventory.carrying += 1;

                        if inventory.carrying > inventory.backpack.capacity {
                            inventory.carrying = inventory.backpack.capacity;
                        }
                    }

                } else{
                    progress.reset();
                }
            }
        }else if self.was_down{
            progress.reset();
        }

        self.was_down = btn_down;
    }
}


/*
fn query_ray(tree: & DynamicBoundingVolumeTree3<f32>, ray: Ray3<f32>) -> Vec<(TreeValueWrapped<Entity, Aabb3<f32>>, Point3<f32>)> {
    let mut visitor = ContinuousVisitor::new(&ray);
    tree.query(&mut visitor)
}
*/



pub type Shape = CollisionShape<Primitive3<f32>, BodyPose3<f32>, Aabb3<f32>, ObjectType>;

#[repr(u8)]
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum ObjectType {
    Box,
}

impl Default for ObjectType {
    fn default() -> Self {
        ObjectType::Box
    }
}

impl Collider for ObjectType {
    fn should_generate_contacts(&self, other: &ObjectType) -> bool {
        true
    }
}

impl Component for ObjectType {
    type Storage = VecStorage<Self>;
}


fn event_was_key_pressed(event: Event,key: VirtualKeyCode)->bool{
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => match virtual_keycode {
                Some(key2) if key == key2 => return true,
                _ => (),
            },
            _ => (),
        },
        _ => (),
    }
    false
}


struct BuyMenuState{
    local_entities: Vec<Entity>,
}

impl State for BuyMenuState{
    fn on_start(&mut self, mut world: &mut World){
        self.local_entities = create_buy_ui(&mut world);
        release_cursor(&mut world.write_resource());
        set_mouse_cursor(&mut world.write_resource(),MouseCursor::Default);
    }

    fn on_stop(&mut self, mut world: &mut World){
        world.delete_entities(self.local_entities.as_slice());
        let dim = world.read_resource::<ScreenDimensions>();
        let mut msg = world.write_resource::<WindowMessages>();

        // Make mouse hidden again
        grab_cursor(&mut msg);
        set_mouse_cursor_none(&mut msg);
    }
    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        if event_was_key_pressed(event,VirtualKeyCode::P){
            println!("Pop buy menu");
            return Trans::Pop;
        }
        Trans::None
    }
}

struct GameState{
    dispatcher: Option<Dispatcher<'static,'static>>,
}

impl GameState{
    pub fn new() -> Self{
        GameState{
            dispatcher: None,
        }
    }
}

pub fn with_bundle<B>(mut disp_builder: DispatcherBuilder<'static,'static>,mut world: &mut World, bundle: B) -> DispatcherBuilder<'static,'static>
    where
        B: ECSBundle<'static, 'static>,
{
    bundle
        .build(&mut world, disp_builder)
        .expect("Failed to add bundle to dispatcher builder")
}


impl State for GameState {
    fn on_start(&mut self, mut world: &mut World) {
        let mut dispatcher = DispatcherBuilder::new()
            .add(MiningSystem::new(),"mining",&[]);
        dispatcher = with_bundle(dispatcher, &mut world,FlyControlBundle::<String, String>::new(
            Some(String::from("move_x")),
            Some(String::from("move_y")),
            Some(String::from("move_z")),
        ).with_speed(20.0).with_sensitivity(0.3,0.3));
        self.dispatcher = Some(dispatcher.build());


        initialise_camera(world);

        let (mut comps,cube) = {
            let mat_defaults = world.read_resource::<MaterialDefaults>().clone();

            let loader = world.read_resource::<Loader>();
            let cube = {
                let mesh_storage = world.read_resource();
                loader.load("cube.obj", ObjFormat, (), (), &mesh_storage)
            };

            let tex_storage = world.read_resource();


            let radius = 4;
            let cube_size = 1.0;

            let mut comps: Vec<(Material, Transform)> = vec![];

            for x in -radius..radius {
                for y in -radius..radius {
                    for z in -radius..radius {

                        // CUBE COLOR
                        let r_color = (x + radius) as f32 / (radius as f32 * 2.0);
                        let g_color = (y + radius) as f32 / (radius as f32 * 2.0);
                        let b_color = (z + radius) as f32 / (radius as f32 * 2.0);

                        let color = mat_from_color([r_color, g_color, b_color, 1.0], &mat_defaults, &loader, &tex_storage);
                        // CUBE COLOR END

                        let x = x as f32 * cube_size;
                        let y = y as f32 * cube_size;
                        let z = z as f32 * cube_size;
                        let mut trans = Transform::default();
                        trans.translation = Vector3::new(x, y, z);

                        comps.push((color, trans));
                    }
                }
            }
            (comps,cube)
        };

        while let Some(c) = comps.pop(){
            world
                .create_entity()
                .with(cube.clone())
                .with(c.0)
                .with(GlobalTransform::default())
                .with_dynamic_rigid_body(
                    Shape::new_simple_with_type(
                        CollisionStrategy::FullResolution,
                        CollisionMode::Discrete,
                        Cuboid::new(1.0, 1.0,1.0).into(),
                        ObjectType::Box,
                    ),
                    BodyPose3::new(Point3::new(c.1.translation.x, c.1.translation.y,c.1.translation.z), Quaternion::one()),
                    Velocity3::from_linear(Vector3::new(0.0,0.0,0.0)),
                    RigidBody::default(),
                    Mass3::new(1.0),
                )
                .with(
                    ForceAccumulator::<Vector3<f32>,Vector3<f32>>::new()
                )
                .with(c.1)
                .build();
        }

        //Plane under the cubes

        let mut trans = Transform::default();
        trans.translation = Vector3::new(0.0, -20.0, 0.0);
        trans.scale = Vector3::new(50.0,5.0,50.0);
        world
            .create_entity()
            .with(GlobalTransform::default())
            .with_static_rigid_body(
                Shape::new_simple_with_type(
                    CollisionStrategy::FullResolution,
                    CollisionMode::Discrete,
                    Cuboid::new(50.0, 5.0,50.0).into(),
                    ObjectType::Box,
                ),
                BodyPose3::new(Point3::new(trans.translation.x, trans.translation.y,trans.translation.z), Quaternion::one()),
                RigidBody::default(),
                Mass3::infinite(),
            )
            .with(trans)
            .build();


        // INVENTORY
        let tool1 = Tool{
            name: String::from("Spoon"),
            icon: load_tool_icon(&world,String::from("Spoon")),
            use_time: 1.0,
            mine_quantity: 1,
            cost: 0,
        };

        let backpack1 = Backpack{
            name: String::from("Hands"),
            icon: load_tool_icon(&world,String::from("Spoon")),
            capacity: 5,
            cost: 0,
        };

        let inventory = Inventory{
            tool: tool1,
            backpack: backpack1,
            carrying: 0,
            money: 0,
        };


        let progress = MineProgress{
            block: None,
            start: 0.0,
            progress: 0.0,
        };



        // World registering stuff

        world.add_resource(inventory);
        world.add_resource(progress);
        world.add_resource(Time::default());
        world.add_resource(AmbientColor(Rgba::from([1.0; 3])));
        world.register::<BlockInstance>();

        // UI

        ui::create_game_ui(&mut world);

    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        /*match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        virtual_keycode,
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => match virtual_keycode {
                    Some(VirtualKeyCode::Escape) => return Trans::Quit,
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }*/

        if event_was_key_pressed(event,VirtualKeyCode::X){
            println!("Push buy menu");
            return Trans::Push(Box::new(BuyMenuState{
                local_entities: Vec::<Entity>::new(),
            }));
        }

        Trans::None
    }

    fn update(&mut self, world: &mut World) -> Trans {
        time_sync(world);
        self.dispatcher.as_mut().unwrap().dispatch(&mut world.res);
        Trans::None
    }
}

fn mat_from_color(color: [f32;4], defaults: &MaterialDefaults, loader: &Loader, tex_storage: &AssetStorage<Texture>)->Material{
    let color = loader.load_from_data(color.into(), (), &tex_storage);
    Material {
        albedo: color,
        ..defaults.0.clone()
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    let resources_directory = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);

    let key_bindings_path = format!(
        "{}/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1.0, 0.6, 0.8, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawUi::new()),
    );

    // PHYSIC-------------------------
    /*let mut channel = EventChannel::<ContactEvent<Entity,Point3<f32>>>::new();
    let reader_2 = channel
        .register_reader();*/

    let mut game = Application::build(resources_directory, GameState::new())?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        /*.with_bundle(FlyControlBundle::<String, String>::new(
            Some(String::from("move_x")),
            Some(String::from("move_y")),
            Some(String::from("move_z")),
        ).with_speed(20.0).with_sensitivity(0.3,0.3))?*/
        .with_bundle(UiBundle::<String,String>::new())?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        .with_bundle(SpatialPhysicsBundle3::<Primitive3<f32>,Aabb3<f32>,ObjectType>::new())?
        .with(UiUpdaterSystem,"ui_updater",&[])
        .with_bundle(TransformBundle::new().with_dep(&["sync_system"]))?
        .build()?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let local = Transform::default();

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .with(GlobalTransform::default())
        .with(FlyControlTag)
        .build();
}
