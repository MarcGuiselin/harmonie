use harmony_modding::prelude::*;

fn init(engine: &mut Harmony) {
    engine.add_feature(TheCube);
}

pub struct TheCube;

impl Feature for TheCube {
    fn build(&self, feature: &mut NewFeature) {
        feature
            .set_name("The Cube")
            .add_systems(Start, start)
            .add_systems(Update, update);
    }
}

fn start(commands: &mut Commands) {
    println!("The time is now: {:?}", std::time::Instant::now());
}
