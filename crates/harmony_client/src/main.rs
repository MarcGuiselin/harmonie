use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_harmonize::ModloaderPlugin))
        .run();
}
