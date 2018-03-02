use specs::{VecStorage,Component};

// tools, equipped, backpack, tool stats

struct Tool{
    pub use_time: f32,
    pub mine_quantity: i32,
    pub cost: i32,
}

struct Backpack{
    pub capacity: i32,
    pub cost: i32,
}

struct BlockInstance{
    pub weight_left: i32,
}

impl Component for BlockInstance{
    type Storage = VecStorage<Self>;
}

struct BlockDefinition{
    pub since_depth: i32,
    pub weight: i32,
}

struct BlockDefinitions{
    blocks: Vec<BlockDefinition>,
}

struct Inventory{
    pub tool: Tool,
    pub backpack: Backpack,
    pub carrying: i32,
}