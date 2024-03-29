use amethyst::core::ecs::prelude::{Component, NullStorage};


/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}