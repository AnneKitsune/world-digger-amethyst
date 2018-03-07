use amethyst::ecs::{VecStorage,Component,System,ReadStorage,Fetch,WriteStorage,Join,Entity};
use amethyst::assets::Handle;
use amethyst::renderer::Texture;
use amethyst::ui::{UiTransform,UiText,UiImage};

// tools, equipped, backpack, tool stats

pub struct Tool{
    pub name: String,
    pub icon: Handle<Texture>,
    pub use_time: f32,
    pub mine_quantity: i32,
    pub cost: i32,
}

pub struct Backpack{
    pub name: String,
    pub icon: Handle<Texture>,
    pub capacity: i32,
    pub cost: i32,
}

pub struct BlockInstance{
    pub weight_left: i32,
}

impl Component for BlockInstance{
    type Storage = VecStorage<Self>;
}

pub struct BlockDefinition{
    pub since_depth: i32,
    pub weight: i32,
}

pub struct BlockDefinitions{
    blocks: Vec<BlockDefinition>,
}

pub struct Inventory{
    pub tool: Tool,
    pub backpack: Backpack,
    pub carrying: i32,
    pub money: i32,
}

pub struct MineProgress{
    pub block: Option<Entity>,
    pub start: f64,
    pub progress: f32,
}

impl MineProgress{
    pub fn reset(&mut self){
        self.block = None;
        self.start = 0.0;
        self.progress = 0.0;
    }
}

pub struct UiUpdaterSystem;

impl<'a> System<'a> for UiUpdaterSystem {
    type SystemData = (
        Fetch<'a, Inventory>,
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, UiImage>,
        Fetch<'a, MineProgress>,
    );

    fn run(&mut self, (inventory,mut transforms,mut texts,mut images,progress): Self::SystemData) {
        for (tr,mut text) in (&transforms,&mut texts).join(){
            match &*tr.id{
                "money" => text.text = format!("{}$",inventory.money),
                "tool" => text.text = format!("{}",inventory.tool.name),
                "backpack" => text.text = format!("{}",inventory.backpack.name),
                "carry" => text.text = format!("{}/{} Kg",inventory.carrying,inventory.backpack.capacity),
                _ => {},
            }
        }

        for (mut tr, mut images) in (&mut transforms,&mut images).join(){
            match &*tr.id{
                "mine progress" => tr.width = progress.progress * 500.0,
                _ => {},
            }
        }
    }
}