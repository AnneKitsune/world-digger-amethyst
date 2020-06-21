

use amethyst::core::{
    bundle::SystemBundle,
    ecs::prelude::{DispatcherBuilder, World}
};
use amethyst::error::Error;
/// Serves all the world gen systems in one package.

#[derive(Debug)]
pub struct WorldGenBundle{}

impl WorldGenBundle {
    pub fn new() -> Self {
        WorldGenBundle {}
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WorldGenBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        //builder.add(ArcBallRotationSystem::default(), "arc_ball_rotation", &[]);
        Ok(())
    }
}