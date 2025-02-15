use api::prelude::*;

pub const SCHEMA: Schema = Mod::new("The cube")
    .add_systems(Start, start)
    .add_systems(Update, update)
    .into_schema();

#[derive(Reflect)]
pub struct Cube;

fn start(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = commands
        .spawn_empty()
        .insert_component(Transform::from_translation(Vec3::ZERO))
        .insert_component(Cube)
        .id();
    println!("Summoned a new entity {:?} with ", entity);
}

fn update() {
    println!("TODO: spin the cube");
}

// From bevy's `examples\3d\3d_shapes.rs`
// fn update(mut query: Query<&mut Transform, With<Cube>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(time.delta_seconds() / 2.);
//     }
// }
