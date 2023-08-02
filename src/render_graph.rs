pub mod shader_builder;

pub struct RenderPassBuilder {}

pub struct RenderPipelineBuilder<'Pipeline> {
    label: Option<&'Pipeline str>,
}

