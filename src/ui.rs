use amethyst::assets::{Loader,AssetStorage,Handle};
use amethyst::core::Parent;
use amethyst::ui::{Anchor, Anchored, DrawUi, FontAsset, MouseReactive, Stretch, Stretched,
                   TtfFormat, UiBundle, UiEvent, UiFocused, UiImage, UiText,
                   UiTransform,TextEditing};
use amethyst::ecs::World;
use amethyst::renderer::Texture;



pub fn create_game_ui(world: &mut World){
    let (font,red,blue,green) = {
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
        let blue = loader.load_from_data([0.0, 0.0, 1.0, 1.0].into(), (), &tex_storage);
        let green = loader.load_from_data([0.0, 1.0, 0.0, 1.0].into(), (), &tex_storage);
        (font,red,blue,green)
    };


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

pub fn load_tool_icon(world: &World, name: String) -> Handle<Texture>{
    let loader = world.read_resource::<Loader>();
    let tex_storage = world.read_resource();
    loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), (), &tex_storage)
}