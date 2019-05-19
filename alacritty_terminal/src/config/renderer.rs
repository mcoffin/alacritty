use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum RendererApi {
    Classic,
    #[cfg(feature = "vulkan")]
    Vulkan
}

impl Default for RendererApi {
    fn default() -> Self {
        RendererApi::Classic
    }
}
