use bevy_app::{App, Plugin};

mod instance;
pub use instance::{ModId, Mods, SystemMods};

mod scheduler;
pub use scheduler::ModSchedules;

pub struct ModloaderPlugin;

impl Plugin for ModloaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mods>()
            .init_resource::<SystemMods>()
            .init_resource::<ModSchedules>();
    }
}
