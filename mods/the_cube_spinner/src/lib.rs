use harmony_modding::prelude::*;

fn init(engine: &mut Harmony) {
    engine.add_feature(TheCubeSpinner);
}

/// An example feature that adds a floating cube spinning in the y-axis
///
/// TODO: build a derive macro for the following impl
///       an idea might be to always make these pub, since we always want to export these
pub struct TheCubeSpinner;

impl StableId {
    const STABLE_ID: &str = "the_cube_spinner|v0.0.0|TheCubeSpinner|entity";
}

impl Feature for TheCube {
    fn build(&self, feature: &mut NewFeature) {
        feature
            .set_name("The Cube")
            .add_systems(Start, start)
            .add_systems(Update, update);
    }
}

fn start(commands: &mut Commands) {
    let entity = commands.spawn_empty().insert().id();
    println!("Summonned a new entity {:?} with ", entity);
}
