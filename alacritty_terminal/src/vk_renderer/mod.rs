use std::marker::PhantomData;
use font::RasterizedGlyph;
use crate::term;
use crate::term::RenderableCell;
use crate::term::color::Rgb;
use crate::renderer::{Glyph, LoadGlyph, GlyphCache};
use crate::renderer::generic;
use crate::renderer::rects::Rects;
use crate::config::Config;
use crate::index::Line;
use glutin::dpi::PhysicalSize;

pub struct VulkanQuadRenderer;

pub struct VulkanRenderApi<'a> {
    data: &'a PhantomData<()>,
}

impl<'a> generic::Renderer for VulkanRenderApi<'a> {
    fn clear(&self, color: Rgb) {
        unimplemented!()
    }

    fn render_string(
        &mut self,
        string: &str,
        line: Line,
        glyph_cache: &mut GlyphCache,
        color: Option<Rgb>
    ) {
        unimplemented!()
    }

    fn render_cell(
        &mut self,
        cell: RenderableCell,
        glyph_cache: &mut GlyphCache
    ) {
        unimplemented!()
    }
}

pub struct VulkanLoaderApi<'a> {
    data: &'a PhantomData<()>,
}

impl<'a> LoadGlyph for VulkanLoaderApi<'a> {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph {
        unimplemented!()
    }

    fn clear(&mut self) {
        unimplemented!()
    }
}

impl generic::BaseRenderContext for VulkanQuadRenderer {
    fn resize(&mut self, size: PhysicalSize, padding_x: f32, padding_y: f32) {
        unimplemented!()
    }
    fn draw_rects(
        &mut self,
        config: &Config,
        props: &term::SizeInfo,
        visual_bell_intensity: f64,
        cell_line_rects: Rects
    ) {
        unimplemented!()
    }
}

impl<'a> generic::RenderContext<'a> for VulkanQuadRenderer {
    type Renderer = VulkanRenderApi<'a>;
    type Loader = VulkanLoaderApi<'a>;

    fn borrow_api(
        &'a mut self,
        config: &'a Config,
        props: &term::SizeInfo
    ) -> Self::Renderer {
        unimplemented!()
    }

    fn borrow_loader(&'a mut self) -> Self::Loader {
        unimplemented!()
    }
}
