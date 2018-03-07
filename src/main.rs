extern crate amethyst;
extern crate amethyst_rhusics;
extern crate rhusics_core;
extern crate rhusics_ecs;
extern crate collision;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{Loader,AssetStorage};
use amethyst::config::Config;
use amethyst::controls::{FlyControlTag,FlyControlBundle};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::core::{Time,Parent};
use amethyst::ecs::{World,VecStorage,Component,Fetch,Entity,System,Join,ReadStorage,FetchMut,Entities};
use amethyst::input::{InputBundle,InputHandler};
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, ElementState, Event,
                         KeyboardInput, Material, MaterialDefaults, MeshHandle, ObjFormat,
                         Pipeline, PosNormTex, Projection, RenderBundle, Rgba, Stage,
                         VirtualKeyCode, WindowEvent,Texture,MouseButton};
use amethyst::shrev::EventChannel;
use amethyst::ui::{Anchor, Anchored, DrawUi, FontAsset, MouseReactive, Stretch, Stretched,
                   TtfFormat, UiBundle, UiEvent, UiFocused, UiImage, UiText,
                   UiTransform,TextEditing};

use amethyst_rhusics::{time_sync, DefaultBasicPhysicsBundle3,SpatialPhysicsBundle3};
use collision::{Aabb3,Ray3};
use collision::dbvt::query_ray_closest;
use collision::primitive::{Primitive3,Cuboid};
use rhusics_core::{CollisionShape, RigidBody,Collider,ContactEvent,Velocity};
use rhusics_ecs::WithRigidBody;
use rhusics_ecs::physics3d::{register_physics,BodyPose3, CollisionMode,
                             CollisionStrategy, Mass3,DynamicBoundingVolumeTree3,SpatialSortingSystem3,ContactEvent3,
                             SpatialCollisionSystem3,GJK3,CurrentFrameUpdateSystem3,NextFrameSetupSystem3,ContactResolutionSystem3};
use amethyst::core::cgmath::{Deg, Array, Basis3,Basis2, One, Point3, Quaternion, Vector3,Matrix3,Zero,EuclideanSpace,Rotation};

mod player;
use player::{Tool,Backpack,BlockDefinition,BlockDefinitions,BlockInstance,Inventory,UiUpdaterSystem,MineProgress};


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
    );

    fn run(&mut self, (entities,tree,camera,transform,block_definitions,input, mut inventory, mut progress, time): Self::SystemData) {
        let btn_down = input.mouse_button_is_down(MouseButton::Left);

        if btn_down {
            for (tr, _) in (&transform, &camera).join() {
                let forward = Quaternion::from(tr.rotation).conjugate().invert() * -Vector3::unit_z();
                let ray = Ray3::new(Point3::from_vec(tr.translation), forward);
                if let Some((v, p)) = query_ray_closest(&*tree, ray) {
                    println!("hit bounding volume of {:?} at point {:?}", v.value, p);

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
        self != other
    }
}

impl Component for ObjectType {
    type Storage = VecStorage<Self>;
}


struct ExampleState;

impl State for ExampleState {
    fn on_start(&mut self, mut world: &mut World) {
        //register_physics::<f32, ObjectType>(&mut world);
        //world.register_physics_3d

        initialise_camera(world);

        let (mut comps,cube,font,red,blue,green) = {
            let mat_defaults = world.read_resource::<MaterialDefaults>().clone();

            let loader = world.read_resource::<Loader>();
            let cube = {
                let mesh_storage = world.read_resource();
                loader.load("cube.obj", ObjFormat, (), (), &mesh_storage)
            };

            let font = loader.load(
                "fonts/Nunito-Black.ttf",
                TtfFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<FontAsset>>(),
            );

            let red = loader.load_from_data([1.0,0.0,0.0,1.0].into(), (), &world.read_resource::<AssetStorage<Texture>>());
            let blue = loader.load_from_data([0.0,0.0,1.0,1.0].into(), (), &world.read_resource::<AssetStorage<Texture>>());
            let green = loader.load_from_data([0.0,1.0,0.0,1.0].into(), (), &world.read_resource::<AssetStorage<Texture>>());

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
            (comps,cube,font,red,blue,green)
        };

        //world.register::<ObjectType>();
        //world.write_resource::<EventChannel<ContactEvent<Entity, Point3<f32>>>>();

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
                    Velocity::<Vector3<f32>,Vector3<f32>>::new(Vector3::new(0.0,-10.0,0.0),Vector3::zero()),
                    RigidBody::default(),
                    Mass3::new(1.0),
                )
                .with(c.1)
                .build();
        }


        /*let background = world
            .create_entity()
            .with(UiTransform::new(
                "background".to_string(),
                0.0,
                0.0,
                0.0,
                20.0,
                20.0,
                0,
            ))
            .with(UiImage {
                texture: red.clone(),
            })
            .with(Stretched::new(Stretch::XY))
            .with(Anchored::new(Anchor::Middle))
            .build();*/

        let mut trans = Transform::default();
        trans.translation = Vector3::new(0.0, -20.0, 0.0);
        world
            .create_entity()
            .with(GlobalTransform::default())
            .with_static_rigid_body(
                Shape::new_simple_with_type(
                    CollisionStrategy::FullResolution,
                    CollisionMode::Discrete,
                    Cuboid::new(50.0, 0.5,50.0).into(),
                    ObjectType::Box,
                ),
                BodyPose3::new(Point3::new(trans.translation.x, trans.translation.y,trans.translation.z), Quaternion::one()),
                RigidBody::default(),
                Mass3::infinite(),
            )
            .with(trans)
            .build();

        world.add_resource(AmbientColor(Rgba::from([1.0; 3])));


        // INVENTORY
        let tool1 = Tool{
            name: String::from("Spoon"),
            icon: red.clone(),
            use_time: 1.0,
            mine_quantity: 1,
            cost: 0,
        };

        let backpack1 = Backpack{
            name: String::from("Hands"),
            icon: blue.clone(),
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
        world.register::<BlockInstance>();

        // UI

        world
            .create_entity()
            .with(UiTransform::new(
                "money".to_string(),
                270.,
                45.,
                1.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "0$".to_string(),
                [0.2, 0.2, 1.0, 1.0],
                50.,
            ))
            .with(Anchored::new(Anchor::TopLeft))
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "tool".to_string(),
                -80.,
                45.,
                1.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "".to_string(),
                [0.2, 0.2, 1.0, 1.0],
                50.,
            ))
            .with(Anchored::new(Anchor::TopRight))
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "backpack".to_string(),
                -80.,
                120.,
                1.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "".to_string(),
                [0.2, 0.2, 1.0, 1.0],
                50.,
            ))
            .with(Anchored::new(Anchor::TopRight))
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "carry".to_string(),
                -80.,
                195.,
                1.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "0 Kg".to_string(),
                [0.2, 0.2, 1.0, 1.0],
                50.,
            ))
            .with(Anchored::new(Anchor::TopRight))
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "mine progress".to_string(),
                0.,
                -50.,
                1.,
                500.,
                32.,
                1,
            ))
            .with(UiImage {
                texture: red.clone(),
            })
            .with(Anchored::new(Anchor::BottomMiddle))
            .build();


        let sell_btn = world
            .create_entity()
            .with(UiTransform::new(
                "sell button".to_string(),
                80.,
                -40.,
                1.,
                150.,
                100.,
                1,
            ))
            .with(UiImage {
                texture: green.clone(),
            })
            .with(Anchored::new(Anchor::MiddleLeft))
            .build();

        world
            .create_entity()
            .with(UiTransform::new(
                "sell text".to_string(),
                0.,
                0.,
                -1.,
                50.,
                40.,
                -1,
            ))
            .with(UiText::new(
                font.clone(),
                "Sell".to_string(),
                [0.0, 0.0, 0.0, 1.0],
                50.,
            ))
            .with(Anchored::new(Anchor::Middle))
            .with(Parent{
                entity: sell_btn,
            })
            .build();

    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
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
                    Some(VirtualKeyCode::Escape) => return Trans::Quit,
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
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

    let mut game = Application::build(resources_directory, ExampleState)?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 0)
        .with_bundle(FlyControlBundle::<String, String>::new(
            Some(String::from("move_x")),
            Some(String::from("move_y")),
            Some(String::from("move_z")),
        ).with_speed(20.0).with_sensitivity(0.3,0.3))?
        .with_bundle(UiBundle::<String,String>::new())?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path),
        )?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?
        //.with_bundle(DefaultBasicPhysicsBundle3::<ObjectType>::new())?
        .with_bundle(SpatialPhysicsBundle3::<Primitive3<f32>,Aabb3<f32>,ObjectType>::new())?
        .with(UiUpdaterSystem,"ui_updater",&[])

        //PHYSICS!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

        /*.with(SpatialSortingSystem3::<f32, BodyPose3<f32>, ObjectType>::new(),"1",&[])
        .with(SpatialCollisionSystem3::<f32, BodyPose3<f32>, ObjectType>::new().with_narrow_phase(GJK3::new()),"2",&["1"])
        .with(CurrentFrameUpdateSystem3::<f32>::new(),"3",&["2"])
        .with(NextFrameSetupSystem3::<f32>::new(),"4",&["3"])
        .with(ContactResolutionSystem3::<f32>::new(reader_2),"5",&["4"])
        .with_resource(EventChannel::<ContactEvent<Entity,Point3<f32>>>::new())
        .with_resource(channel)*/
        .with(MiningSystem::new(),"mining",&[])
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