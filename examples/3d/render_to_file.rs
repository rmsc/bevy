use bevy::{
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera, CameraProjection},
        pass::*,
        render_graph::{
            base::MainPass, CameraNode, PassNode, RenderGraph, TextureNode, TextureReadoutNode,
        },
        texture::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage},
    },
    window::WindowId,
};

/// This example renders a second camera to a texture and saves it to a file
fn main() {
    App::build()
        //.add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    mut active_cameras: ResMut<ActiveCameras>,
    mut render_graph: ResMut<RenderGraph>,
    asset_server: Res<AssetServer>,
    msaa: Res<Msaa>,
) {
    let size = Extent3d::new(512, 512, 1);

    // setup the render graph to draw a second camera to a texture, and then to a file

    // add a new camera
    render_graph.add_system_node("secondary_camera", CameraNode::new("Secondary"));

    // add a new resource handle for the texture
    //let texture_handle = HandleUntyped::<Texture>::weak();

    // add a texture node for the second camera
    render_graph.add_node(
        "save_to_file_texture",
        TextureNode::new(
            TextureDescriptor {
                size: size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: Default::default(),
                usage: TextureUsage::OUTPUT_ATTACHMENT
                    | TextureUsage::COPY_SRC,
            },
        ),
    );

    // add a depth texture for the second camera
    render_graph.add_node(
        "save_to_file_depth_texture",
        TextureNode::new(
            TextureDescriptor {
                size: size,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT
                     | TextureUsage::COPY_SRC,
                sample_count: msaa.samples,
                ..Default::default()
            },
        ),
    );

    // add a new render pass for our new camera and texture
    let mut save_to_file_pass = PassNode::<&MainPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment_descriptor(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Clear(Color::rgb(0.5, 0.5, 0.8)),
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });
    save_to_file_pass.add_camera("Secondary");
    active_cameras.add("Secondary");

    render_graph.add_node("save_to_file_pass", save_to_file_pass);

    render_graph
        .add_slot_edge(
            "save_to_file_texture",
            TextureNode::OUT_TEXTURE,
            "save_to_file_pass",
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    render_graph
        .add_slot_edge(
            "save_to_file_depth_texture",
            TextureNode::OUT_TEXTURE,
            "save_to_file_pass",
            "depth",
        )
        .unwrap();

    render_graph
        .add_node_edge("secondary_camera", "save_to_file_pass")
        .unwrap();

    if msaa.samples > 1 {
       render_graph.add_node(
            "second_multi_sampled_color_attachment",
            TextureNode::new(
                TextureDescriptor {
                    size: size,
                    mip_level_count: 1,
                    sample_count: msaa.samples,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::default(),
                    usage: TextureUsage::OUTPUT_ATTACHMENT
                        | TextureUsage::COPY_SRC,
                },
            ),
        );

        render_graph
            .add_slot_edge(
                "second_multi_sampled_color_attachment",
                TextureNode::OUT_TEXTURE,
                "save_to_file_pass",
                "color_attachment",
            )
            .unwrap();
    }

    // create a closure to save the file
    let file_saver = |data: &[u8], descriptor: TextureDescriptor| {
        match descriptor.format {
            TextureFormat::Bgra8UnormSrgb => {
                image::save_buffer(
                    "test.jpg",
                    data.as_ref(),
                    descriptor.size.width,
                    descriptor.size.height,
                    image::ColorType::Bgra8,
                )
                    .unwrap();
                }
            _ => {}
        }
    };

    // add a texture readout node
    render_graph.add_node(
        "save_to_file_readout",
        TextureReadoutNode::new(
            TextureDescriptor {
                size: size,
                format: Default::default(),
                ..Default::default()
            },
            file_saver,
        ));

    // set the correct texture as the input to the readout node
    render_graph
        .add_slot_edge(
            "save_to_file_texture",
            TextureNode::OUT_TEXTURE,
            "save_to_file_readout",
            TextureReadoutNode::IN_TEXTURE,
        )
        .unwrap();

    // make sure the readout node waits for the render pass to finish
    render_graph
        .add_node_edge("save_to_file_pass", "save_to_file_readout")
        .unwrap();

    // SETUP SCENE

    commands
        .spawn_scene(asset_server.load("models/monkey/Monkey.gltf#Scene0"))
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 5.0, 4.0),
            ..Default::default()
        })
        // main camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 6.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });

        // save to file camera, hack around not having a window
        let mut secondary_camera = PerspectiveCameraBundle {
            camera: Camera {
                name: Some("Secondary".to_string()),
                window: WindowId::new(),
                ..Default::default()
            },
            transform: Transform::from_xyz(6.0, 0.0, 0.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        };
        let camera_projection = &mut secondary_camera.perspective_projection;
        camera_projection.update(size.width as f32, size.height as f32);
        secondary_camera.camera.projection_matrix = camera_projection.get_projection_matrix();
        secondary_camera.camera.depth_calculation = camera_projection.depth_calculation();
        commands.spawn(secondary_camera);
}
