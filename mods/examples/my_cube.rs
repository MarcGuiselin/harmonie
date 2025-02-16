use api::prelude::*;

pub const SCHEMA: Schema = Mod::new("My cube")
    .add_resource::<CountFrames>()
    .add_systems(Start, spawn_cube)
    .add_systems(Update, (update_frame_count, rotate_cube).chain())
    .into_schema();

/// A marker component for our Cube
#[derive(Reflect)]
pub struct MyCube;

fn spawn_cube(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let mesh = meshes.add(Cuboid::default());
    // let material = materials.add(StandardMaterial::default());

    let entity = commands
        .spawn_empty()
        // .insert_component(Name("My cube".into()))
        // .insert_component(Mesh3d(mesh))
        // .insert_component(MeshMaterial3d(material))
        .insert_component(Transform::from_translation(Vec3::ZERO))
        .insert_component(MyCube)
        .id();

    println!("Summoned my cube as {:?}", entity);
}

#[derive(Reflect, Default)]
pub struct CountFrames(u32);

fn update_frame_count(mut resource: ResMut<CountFrames>) {
    println!("Frame {}", resource.0);
    resource.0 += 1;
}

// From bevy's `examples\3d\3d_shapes.rs`
fn rotate_cube(// mut query: Query<&mut Transform, With<MyCube>>,
    // time: Res<Time>
) {
    // for mut transform in &mut query {
    //     transform.rotate_y(time.delta_seconds() / 2.);
    // }
}
