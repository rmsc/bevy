use bevy::{
    math::clamp,
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera},
        render_graph::{base::MainPass, CameraNode, PassNode, RenderGraph},
        surface::{SideLocation, Viewport, ViewportDescriptor},
    },
};

/// This example creates a second window and draws a mesh from two different cameras.
fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<ViewportLayout>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(viewport_layout_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    mut active_cameras: ResMut<ActiveCameras>,
    mut render_graph: ResMut<RenderGraph>,
    asset_server: Res<AssetServer>,
) {
    // add new camera nodes for the secondary viewports
    render_graph.add_system_node("top_right_camera", CameraNode::new("TopRight"));
    render_graph.add_system_node("bottom_right_camera", CameraNode::new("BottomRight"));
    active_cameras.add("TopRight");
    active_cameras.add("BottomRight");

    // add the cameras to the main pass
    {
        let main_pass: &mut PassNode<&MainPass> = render_graph.get_node_mut("main_pass").unwrap();
        main_pass.add_camera("TopRight");
        main_pass.add_camera("BottomRight");
    }
    render_graph
        .add_node_edge("top_right_camera", "main_pass")
        .unwrap();
    render_graph
        .add_node_edge("bottom_right_camera", "main_pass")
        .unwrap();

    // SETUP SCENE

    // add entities to the world
    commands
        //.spawn_scene(asset_server.load("models/monkey/Monkey.gltf#Scene0"))
        .spawn_scene(asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"))
        // light
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 5.0, 4.0),
            ..Default::default()
        })
        // main camera
        .spawn(PerspectiveCameraBundle {
            // the following is an example of how to setup static viewports
            // and isn't really necessary in this case, as it will be
            // immediately overwritten by the viewport_layout_system
            viewport: Viewport::new(ViewportDescriptor {
                sides: Rect {
                    // occupy the left 50% of the available horizontal space
                    left: SideLocation::Relative(0.0),
                    right: SideLocation::Relative(0.5),
                    // occupy the left 100% of the available vertical space
                    top: SideLocation::Relative(0.0),
                    bottom: SideLocation::Relative(1.0),
                },
                ..Default::default()
            }),
            transform: Transform::from_xyz(-1.0, 1.0, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::unit_y()),
            ..Default::default()
        })
        // top right camera
        .spawn(PerspectiveCameraBundle {
            camera: Camera {
                name: Some("TopRight".to_string()),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.3, 1.3)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::unit_y()),
            ..Default::default()
        })
        // bottom right camera
        .spawn(PerspectiveCameraBundle {
            camera: Camera {
                name: Some("BottomRight".to_string()),
                ..Default::default()
            },
            transform: Transform::from_xyz(-1.3, 0.3, 0.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::unit_y()),
            ..Default::default()
        });

    // ui
    let instructions_text = "use the arrow keys to resize the viewports";
    commands
        .spawn(UiCameraBundle {
            // viewports occupy the entire surface by default, and can overlap each other
            ..Default::default()
        })
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text::with_section(
                instructions_text,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        });
}

struct ViewportLayout {
    divide_x: f32,
    divide_y: f32,
}

impl Default for ViewportLayout {
    fn default() -> Self {
        Self {
            divide_x: 0.5,
            divide_y: 0.5,
        }
    }
}

fn viewport_layout_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut layout: ResMut<ViewportLayout>,
    mut query: Query<(&Camera, &mut Viewport)>,
) {
    // update the layout state
    if keyboard_input.just_pressed(KeyCode::Left) {
        layout.divide_x -= 0.05;
    }
    if keyboard_input.just_pressed(KeyCode::Right) {
        layout.divide_x += 0.05;
    }
    if keyboard_input.just_pressed(KeyCode::Up) {
        layout.divide_y -= 0.05;
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        layout.divide_y += 0.05;
    }
    layout.divide_x = clamp(layout.divide_x, 0.0, 1.0);
    layout.divide_y = clamp(layout.divide_y, 0.0, 1.0);

    // resize the viewports
    for (camera, mut viewport) in query.iter_mut() {
        match camera.name.as_deref() {
            // default camera
            Some("Camera3d") => {
                viewport.sides.right = SideLocation::Relative(layout.divide_x);
            }
            Some("TopRight") => {
                viewport.sides.left = SideLocation::Relative(layout.divide_x);
                viewport.sides.bottom = SideLocation::Relative(layout.divide_y);
            }
            Some("BottomRight") => {
                viewport.sides.left = SideLocation::Relative(layout.divide_x);
                viewport.sides.top = SideLocation::Relative(layout.divide_y);
            }
            _ => {}
        }
    }
}