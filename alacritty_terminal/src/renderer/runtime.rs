use crate::term::RenderableCell;
use crate::index::Line;
use super::QuadRenderer;
use crate::vk_renderer::VulkanQuadRenderer;
use super::generic;
use super::{LoadGlyph, GlyphCache};
use super::rects::Rects;
use super::generic::{BaseRenderContext, RenderContext, Renderer};
use crate::term;
use crate::config::Config;
use glutin::dpi::PhysicalSize;
use crate::term::color::Rgb;
use font::RasterizedGlyph;
use crate::renderer::Glyph;

pub enum RuntimeRenderer<'a> {
    Classic(<QuadRenderer as RenderContext<'a>>::Renderer),
    Vulkan(<VulkanQuadRenderer as RenderContext<'a>>::Renderer),
}

impl<'a> RuntimeRenderer<'a> {
    fn unwrap(&self) -> &dyn Renderer {
        use RuntimeRenderer::*;
        match self {
            &Classic(ref r) => r,
            &Vulkan(ref r) => r,
        }
    }
    fn unwrap_mut(&mut self) -> &mut dyn Renderer {
        use RuntimeRenderer::*;
        match self {
            &mut Classic(ref mut r) => r,
            &mut Vulkan(ref mut r) => r,
        }
    }
}

impl<'a> Renderer for RuntimeRenderer<'a> {
    fn clear(&self, color: Rgb) {
        self.unwrap().clear(color);
    }

    fn render_string(
        &mut self,
        string: &str,
        line: Line,
        glyph_cache: &mut GlyphCache,
        color: Option<Rgb>
    ) {
        self.unwrap_mut().render_string(string, line, glyph_cache, color)
    }

    fn render_cell(&mut self, cell: RenderableCell, glyph_cache: &mut GlyphCache) {
        self.unwrap_mut().render_cell(cell, glyph_cache);
    }
}

pub enum RuntimeLoader<'a> {
    Classic(<QuadRenderer as RenderContext<'a>>::Loader),
    Vulkan(<VulkanQuadRenderer as RenderContext<'a>>::Loader),
}

impl<'a> RuntimeLoader<'a> {
    fn unwrap_mut(&mut self) -> &mut dyn LoadGlyph {
        use RuntimeLoader::*;
        match self {
            &mut Classic(ref mut r) => r,
            &mut Vulkan(ref mut r) => r,
        }
    }
}

impl<'a> LoadGlyph for RuntimeLoader<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        self.unwrap_mut().load_glyph(rasterized)
    }

    fn clear(&mut self) {
        self.unwrap_mut().clear()
    }
}

pub enum RuntimeQuadRenderer {
    Classic(QuadRenderer),
    Vulkan(VulkanQuadRenderer)
}

impl generic::BaseRenderContext for RuntimeQuadRenderer {
    fn resize(&mut self, size: PhysicalSize, padding_x: f32, padding_y: f32) {
        use RuntimeQuadRenderer::*;
        match self {
            &mut Classic(ref mut r) => r.resize(size, padding_x, padding_y),
            &mut Vulkan(ref mut r) => r.resize(size, padding_x, padding_y)
        }
    }

    fn draw_rects(
        &mut self,
        config: &Config,
        props: &term::SizeInfo,
        visual_bell_intensity: f64,
        cell_line_rects: Rects
    ) {
        use RuntimeQuadRenderer::*;
        let renderer_dyn: &mut dyn BaseRenderContext = match self {
            &mut Classic(ref mut r) => r,
            &mut Vulkan(ref mut r) => r,
        };
        renderer_dyn.draw_rects(config, props, visual_bell_intensity, cell_line_rects)
    }
}

impl<'a> generic::RenderContext<'a> for RuntimeQuadRenderer {
    type Renderer = RuntimeRenderer<'a>;
    type Loader = RuntimeLoader<'a>;

    fn borrow_api(
        &'a mut self,
        config: &'a Config,
        props: &term::SizeInfo
    ) -> Self::Renderer {
        match self {
            &mut RuntimeQuadRenderer::Classic(ref mut r) => RuntimeRenderer::Classic(r.borrow_api(config, props)),
            &mut RuntimeQuadRenderer::Vulkan(ref mut r) => RuntimeRenderer::Vulkan(r.borrow_api(config, props)),
        }
    }

    fn borrow_loader(&'a mut self) -> Self::Loader {
        match self {
            &mut RuntimeQuadRenderer::Classic(ref mut r) => RuntimeLoader::Classic(r.borrow_loader()),
            &mut RuntimeQuadRenderer::Vulkan(ref mut r) => RuntimeLoader::Vulkan(r.borrow_loader())
        }
    }
}
