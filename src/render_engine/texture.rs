use crate::render_engine::DeviceState;
use crate::resource::{ ResourceHandler, ResourceMetaData };
use uuid::Uuid;

pub struct Texture {
    id: Uuid,
    texture: wgpu::Texture,
    view: wgpu::TextureView
}

pub struct TextureHandler<'manager> {
    device_state: &'manager DeviceState,
    surface_texture: Option<Texture>
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
        self.surface_texture = Some(Texture {
            id: Uuid::new_v4(),
            view: surface_view,
            texture: surface_texture.texture,
        });
        self.surface_texture.unwrap().id
    }
}

impl ResourceHandler<Texture> for TextureHandler<'_> {
    fn create(&mut self, meta_data: &ResourceMetaData) -> Texture {
        let is_surface = if let Some(surface) = self.surface_texture {
            meta_data.uuid == surface.id
        } else {
            false
        };

        return self.surface_texture.take().unwrap()
    }

    fn destroy(&mut self, texture: Texture) {

    }
}

