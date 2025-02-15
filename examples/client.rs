use bevy::prelude::*;
use bevy_harmonize::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ModloaderPlugin))
        .run();
}
