use crate::config::Config;
use crate::term;
use crate::term::RenderableCell;
use crate::term::color::Rgb;
use crate::index::Line;
use super::GlyphCache;
use super::LoadGlyph;
use super::rects::Rects;
use glutin::dpi::PhysicalSize;

pub trait RenderContext<'a> {
    type Renderer: Renderer;
    type Loader: LoadGlyph;

    fn with_api<'config, F, T>(
        self,
        config: &'config Config,
        props: &term::SizeInfo,
        func: F
    ) -> T where
        'config: 'a,
        F: FnOnce(Self::Renderer) -> T;

    fn with_loader<F, T>(
        self,
        func: F
    ) -> T where
        F: FnOnce(Self::Loader) -> T;

    fn resize(self, size: PhysicalSize, padding_x: f32, padding_y: f32);

    fn draw_rects(
        self,
        config: &Config,
        props: &term::SizeInfo,
        visual_bell_intensity: f64,
        cell_line_rects: Rects
    );
}

pub trait Renderer {
    fn clear(&self, color: Rgb);
    fn render_string(
        &mut self,
        string: &str,
        line: Line,
        glyph_cache: &mut GlyphCache,
        color: Option<Rgb>
    );
    fn render_cell(
        &mut self,
        cell: RenderableCell,
        glyph_cache: &mut GlyphCache
    );
}
