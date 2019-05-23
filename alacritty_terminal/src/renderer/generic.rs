use glutin::dpi::PhysicalSize;
use font::RasterizedGlyph;

use crate::config::{ self, Config };
use crate::index::Line;
use crate::term::{ self, RenderableCell };
use crate::term::color::Rgb;
use super::rects::Rects;

pub trait RenderApi {
    fn clear(&self, color: Rgb);
    fn render_string(
        &mut self,
        string: &str,
        line: Line,
        color: Option<Rgb>
    );
    fn render_cell(
        &mut self,
        cell: RenderableCell
    );
}

pub trait LoaderApi {
    type Glyph;
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Self::Glyph;
    fn clear(&mut self);
}

pub trait Renderer<'a> {
    type LoaderApi: LoaderApi;
    type RenderApi: RenderApi + LoaderApi<Glyph=<Self::LoaderApi as LoaderApi>::Glyph>;

    fn cell_dimensions(&self, config: &Config) -> (f32, f32);
    fn resize(&mut self, size: PhysicalSize, padding_x: f32, padding_y: f32);

    fn update_font_size(
        &mut self,
        font: &config::Font,
        size: font::Size,
        dpr: f64
    ) -> Result<(), font::Error>;
    fn font_metrics(&self) -> font::Metrics;

    fn draw_rects(
        &mut self,
        config: &Config,
        props: &term::SizeInfo,
        visual_bell_intensity: f64,
        cell_line_rects: Rects
    );

    unsafe fn borrow_api(
        &'a mut self,
        config: &'a Config,
        props: &'a term::SizeInfo
    ) -> Self::RenderApi;

    unsafe fn borrow_loader(&'a mut self) -> Self::LoaderApi;

    fn with_api<F, T>(
        &'a mut self,
        config: &'a Config,
        props: &'a term::SizeInfo,
        func: F
    ) -> T where
        F: FnOnce(Self::RenderApi) -> T,
    {
        let api = unsafe { self.borrow_api(config, props) };
        func(api)
    }

    fn with_loader<F, T>(
        &'a mut self,
        func: F
    ) -> T where
        F: FnOnce(Self::LoaderApi) -> T,
    {
        let loader = unsafe { self.borrow_loader() };
        func(loader)
    }
}
