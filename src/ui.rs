use amethyst::assets::{Loader,AssetStorage,Handle};
use amethyst::core::Parent;
use amethyst::ui::{Anchor, Anchored, DrawUi, FontAsset, MouseReactive, Stretch, Stretched,
                   TtfFormat, UiBundle, UiEvent, UiFocused, UiImage, UiText,
                   UiTransform,TextEditing};
use amethyst::ecs::{World,Entity,Component,VecStorage,FetchMut};
use amethyst::renderer::Texture;

#[derive(Clone)]
pub struct UiShit{
    font: Handle<FontAsset>,
    red: Handle<Texture>,
    green: Handle<Texture>,
    blue: Handle<Texture>,
    gray_overlay: Handle<Texture>,
    brown: Handle<Texture>,
}

impl Component for UiShit{
    type Storage = VecStorage<Self>;
}

pub fn load_ui_shit(world: &World) -> UiShit{
    let loader = world.read_resource::<Loader>();
    let font = loader.load(
        "fonts/Nunito-Black.ttf",
        TtfFormat,
        Default::default(),
        (),
        &world.read_resource::<AssetStorage<FontAsset>>(),
    );
    let tex_storage = world.read_resource();
    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage);
    let green = loader.load_from_data([0.0, 1.0, 0.0, 1.0].into(), (), &tex_storage);
    let blue = loader.load_from_data([0.0, 0.0, 1.0, 1.0].into(), (), &tex_storage);
    let gray_overlay = loader.load_from_data([0.8, 0.8, 0.8, 0.8].into(), (), &tex_storage);
    let brown = loader.load_from_data([0.54, 0.27, 0.07, 1.0].into(), (), &tex_storage);

    UiShit{
        font,
        red,
        green,
        blue,
        gray_overlay,
        brown,
    }
}

pub fn fetch_ui(mut world: &mut World) -> UiShit{
    if let Some(ui) = world.res.try_fetch::<UiShit>(0){
        return ui.clone();
    }
    let ui = load_ui_shit(&world);
    world.add_resource(ui.clone());
    ui
}

pub fn create_game_ui(mut world: &mut World){
    let ui = fetch_ui(&mut world);
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
            ui.font.clone(),
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
            ui.font.clone(),
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
            ui.font.clone(),
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
            ui.font.clone(),
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
            texture: ui.red.clone(),
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
            texture: ui.green.clone(),
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
            ui.font.clone(),
            "Sell".to_string(),
            [0.0, 0.0, 0.0, 1.0],
            50.,
        ))
        .with(Anchored::new(Anchor::Middle))
        .with(Stretched::new(Stretch::XY))
        .with(Parent{
            entity: sell_btn,
        })
        .build();
}


pub fn create_buy_ui(mut world: &mut World)->Vec<Entity>{
    let ui = fetch_ui(&mut world);
    let mut out = Vec::<Entity>::new();

    let bg = world
        .create_entity()
        .with(UiTransform::new(
            "shopbg".to_string(),
            0.,
            0.,
            -10.,
            0.,
            0.,
            1,
        ))
        .with(UiImage {
            texture: ui.gray_overlay.clone(),
        })
        .with(Anchored::new(Anchor::Middle))
        .with(Stretched::new(Stretch::XY))
        .build();

    let tool_bg = world
        .create_entity()
        .with(UiTransform::new(
            "toolbg".to_string(),
            270.,
            0.,
            -1.,
            500.,
            0.,
            1,
        ))
        .with(UiImage {
            texture: ui.brown.clone(),
        })
        .with(Anchored::new(Anchor::MiddleLeft))
        .with(Stretched::new(Stretch::Y).with_margin(0.0,20.0))
        .with(Parent{
            entity: bg.clone(),
        })
        .build();

    out.push(world
        .create_entity()
        .with(UiTransform::new(
            "shop".to_string(),
            0.,
            0.,
            -1.,
            500.,
            75.,
            1,
        ))
        .with(UiText::new(
            ui.font.clone(),
            "shop".to_string(),
            [0.2, 0.2, 1.0, 1.0],
            50.,
        ))
        .with(Anchored::new(Anchor::Middle))
        .with(Parent{
            entity: bg.clone(),
        })
        .build());

    println!("tool_bg: {:?}",tool_bg);

    out.push(bg);
    out.push(tool_bg);

    out
}


pub fn load_tool_icon(world: &World, name: String) -> Handle<Texture>{
    let loader = world.read_resource::<Loader>();
    let tex_storage = world.read_resource();
    loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage)
}