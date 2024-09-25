use api::prelude::*;

pub fn init(engine: &mut Harmony) {
    engine.add_feature(TheCubeSpinner);
}

/// An example feature that adds a floating cube spinning in the y-axis
///
/// TODO: build a derive macro for the following impl
///       an idea might be to always make these pub, since we always want to export these
pub struct TheCubeSpinner;

impl HasStableId for TheCubeSpinner {
    /// This is how identical features are identified between mods
    const CRATE_NAME: &'static str = "the_cube";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "TheCube";
}

impl Feature for TheCubeSpinner {
    fn build(&self, feature: &mut FeatureBuilder) {
        feature
            .set_name("The Cube Spinner")
            .add_systems(Update, update);
    }
}

fn update() {
    println!("TODO: move cube");
}
