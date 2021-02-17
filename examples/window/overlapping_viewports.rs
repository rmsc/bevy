use bevy::{
    math::Rect,
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera},
        render_graph::{base::MainPass, CameraNode, PassNode, RenderGraph},
        surface::{SideLocation, Viewport, ViewportDescriptor},
    },
};

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut render_graph: ResMut<RenderGraph>,
) {
    render_graph.add_system_node("overlap_camera", CameraNode::new("Overlap"));
    active_cameras.add("Overlap");
    {
        let main_pass: &mut PassNode<&MainPass> = render_graph.get_node_mut("main_pass").unwrap();
        main_pass.add_camera("Overlap");
    }
    render_graph
        .add_node_edge("overlap_camera", "main_pass")
        .unwrap();

    // add entities to the world
    commands
        // plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        // light
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .spawn(PerspectiveCameraBundle {
            camera: Camera {
                name: Some("Overlap".to_string()),
                ..Default::default()
            },
            viewport: Viewport::new(ViewportDescriptor {
                sides: Rect {
                    left: SideLocation::Relative(0.65),
                    right: SideLocation::Relative(0.9),
                    top: SideLocation::Relative(0.65),
                    bottom: SideLocation::Relative(0.9),
                },
                // this will treat the overlapping viewport as if it is
                // a flat plane at depth zero, so above everything else
                depth_range: 0.0..=0.0,
                ..Default::default()
            }),
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
