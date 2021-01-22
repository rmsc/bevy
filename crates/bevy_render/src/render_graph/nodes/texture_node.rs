use crate::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{RenderContext, RenderResourceId, RenderResourceType},
    texture::TextureDescriptor,
};
use bevy_ecs::{Resources, World};
use std::borrow::Cow;

pub struct TextureNode {
    descriptor: TextureDescriptor,
}

impl TextureNode {
    pub const OUT_TEXTURE: &'static str = "texture";

    pub fn new(descriptor: TextureDescriptor) -> Self {
        Self {
            descriptor
        }
    }
}

impl Node for TextureNode {
    fn output(&self) -> &[ResourceSlotInfo] {
        static OUTPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(TextureNode::OUT_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        OUTPUT
    }

    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        output: &mut ResourceSlots,
    ) {
        const TEXTURE: usize = 0;
        if output.get(TEXTURE).is_none() {
            let render_resource_context = render_context.resources_mut();
            let texture_resource = render_resource_context.create_texture(self.descriptor);
            output.set(TEXTURE, RenderResourceId::Texture(texture_resource));
        }
    }
}
