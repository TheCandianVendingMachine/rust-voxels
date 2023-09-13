use crate::render_engine::DeviceState;
use crate::resource::{ ResourceHandler, ResourceMetaData };
use uuid::Uuid;
use std::sync::Arc;

struct Surface {
    id: Uuid,
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView
}

struct Dynamic {
    id: Uuid,
    texture: wgpu::Texture,
    view: wgpu::TextureView
}

pub enum Texture {
    None,
    Surface(Arc<Surface>),
    Dynamic(Dynamic)
}

pub struct TextureHandler<'manager> {
    device_state: &'manager DeviceState,
    surface_texture: Option<Arc<Surface>>
}

impl<'manager> TextureHandler<'manager> {
    pub fn new(device_state: &'manager DeviceState) -> TextureHandler {
        TextureHandler {
            device_state,
            surface_texture: None
        }
    }

    pub fn set_surface(&mut self, surface: &wgpu::Surface) -> Uuid {
        let surface_texture = surface.get_current_texture().unwrap();
        let surface_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = Uuid::new_v4();
        self.surface_texture = Some(Arc::new(Surface {
            id,
            texture: surface_texture,
            view: surface_view
        }));
        id
    }
}

impl ResourceHandler<Texture> for TextureHandler<'_> {
    fn create(&mut self, meta_data: &ResourceMetaData) -> Texture {
        let is_surface = if let Some(surface) = &self.surface_texture {
            meta_data.uuid == surface.id
        } else {
            false
        };

        Texture::Surface(self.surface_texture.as_ref().unwrap().clone())
    }

    fn destroy(&mut self, texture: Texture) {

    }
}

