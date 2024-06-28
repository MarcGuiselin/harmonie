use harmony_modding::prelude::*;

fn init(engine: &mut Harmony) {
    engine.add_feature(TheCube);
}

/// An example feature that adds a floating cube spinning in the y-axis
///
/// TODO: build a `Feature` derive macro for the following impl
///       an idea might be for the macro to always make these pub, since we always want to export these for other mods
pub struct TheCube;

impl StableId for TheCube {
    /// This is how identical features are identified between mods
    const CRATE_NAME: &'static str = "the_cube";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "TheCube";
}

impl Feature for TheCube {
    fn build(&self, feature: &mut NewFeature) {
        feature
            .set_name("The Cube")
            .add_systems(Start, start)
            .add_systems(Update, update);
    }
}

// TODO: build a `Resource` derive macro for the following impl
#[derive(bitcode::Decode, bitcode::Encode)]
struct Cube;

impl StableId for Cube {
    const CRATE_NAME: &'static str = "the_cube";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Cube";
}

static mut LOCAL_COMPONENT_ID_FOR_CUBE: Option<u32> = None;
impl Component for Cube {
    fn get_local_component_id() -> u32 {
        #[link(wasm_import_module = "harmony_mod")]
        extern "C" {
            pub fn reserve_component_id() -> u32;
        }

        // SAFETY: Mods run single-threaded
        unsafe {
            LOCAL_COMPONENT_ID_FOR_CUBE.unwrap_or_else(|| {
                let id = reserve_component_id();
                LOCAL_COMPONENT_ID_FOR_CUBE = Some(id);
                id
            })
        }
    }
}

fn start(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = commands
        .spawn_empty()
        //.insert_component(Transform)
        .insert_component(Cube)
        .id();
    println!("Summonned a new entity {:?} with ", entity);
}

// From bevy's `examples\3d\3d_shapes.rs`
// fn update(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(time.delta_seconds() / 2.);
//     }
// }
