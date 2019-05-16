use std::borrow::BorrowMut;
use crate::config::Config;
use crate::term;
use crate::term::RenderableCell;
use crate::term::color::Rgb;
use crate::index::Line;
use super::GlyphCache;
use super::LoadGlyph;
use super::rects::Rects;
use glutin::dpi::PhysicalSize;

pub trait BaseRenderContext {
    fn resize(&mut self, size: PhysicalSize, padding_x: f32, padding_y: f32);
    fn draw_rects(
        &mut self,
        config: &Config,
        props: &term::SizeInfo,
        visual_bell_intensity: f64,
        cell_line_rects: Rects
    );
}

pub trait RenderContext<'a>: BaseRenderContext {
    type Renderer: Renderer;
    type Loader: LoadGlyph;

    fn borrow_api(
        &'a mut self,
        config: &'a Config,
        props: &term::SizeInfo
    ) -> Self::Renderer;

    fn borrow_loader(&'a mut self) -> Self::Loader;

    fn with_api<'me, 'config, F, T>(
        &'me mut self,
        config: &'config Config,
        props: &term::SizeInfo,
        func: F
    ) -> T where
        'me: 'a,
        'config: 'a,
        F: FnOnce(Self::Renderer) -> T,
    {
        let api = self.borrow_api(config, props);
        func(api)
    }

    fn with_loader<'me, F, T>(
        &'me mut self,
        func: F
    ) -> T where
        'me: 'a,
        F: FnOnce(Self::Loader) -> T,
    {
        let api = self.borrow_loader();
        func(api)
    }
}

pub trait DynamicRenderContext: BaseRenderContext {
    fn with_api_dynamic<F, T>(
        &mut self,
        config: &Config,
        props: &term::SizeInfo,
        func: F
    ) -> T where
        F: FnOnce(&mut dyn Renderer) -> T;

    fn with_loader_dynamic<F, T>(
        &mut self,
        func: F
    ) -> T where
        F: FnOnce(&mut dyn LoadGlyph) -> T;
}

//impl<R> DynamicRenderContext for R where
//    for<'a> R: RenderContext<'a>,
//{
//    fn with_api_dynamic<F, T>(
//        &mut self,
//        config: &Config,
//        props: &term::SizeInfo,
//        func: F
//    ) -> T where
//        F: FnOnce(&mut dyn Renderer) -> T,
//    {
//        self.with_api(config, props, |mut api| func(&mut api))
//    }
//
//    fn with_loader_dynamic<F, T>(
//        &mut self,
//        func: F
//    ) -> T where
//        F: FnOnce(&mut dyn LoadGlyph) -> T,
//    {
//        self.with_loader(|mut api| func(&mut api))
//    }
//}

//pub struct DynamicRenderer<R> {
//    renderer: R
//}
//
//impl<R> DynamicRenderer<R> where for<'a> R: RenderContext<'a> {
//    fn new(renderer: R) -> DynamicRenderer<R> {
//        DynamicRenderer {
//            renderer: renderer
//        }
//    }
//}
//
//impl<'a> RenderContext<'a> for DynamicRenderer<R> where
//    R: RenderContext<'a>,
//{
//}

//pub struct DynamicRenderer<R> where {
//    pub renderer: R
//}
//
//impl<'a, R> RenderContext<'a> for &'a mut DynamicRenderer<R> where
//    &'a mut R: RenderContext<'a>
//{
//    type Renderer = &'a mut Renderer;
//    type Loader = &'a mut LoadGlyph;
//
//    fn with_api<'config, F, T>(
//        self,
//        config: &'config Config,
//        props: &term::SizeInfo,
//        func: F
//    ) -> T where
//        'config: 'a,
//        F: FnOnce(Self::Renderer) -> T,
//    {
//        //let r: &'a mut dyn RenderContext<'a, Renderer = <R as RenderContext<'a>>::Renderer, Loader = <R as RenderContext<'a>>::Loader> = &self.renderer;
//        self.renderer.with_api(config, props, |api| func(&mut api))
//    }
//
//    fn with_loader<F, T>(
//        self,
//        func: F
//    ) -> T where
//        F: FnOnce(Self::Loader) -> T,
//    {
//        self.renderer.with_loader(|api| func(&mut api))
//    }
//
//    fn resize(self, size: PhysicalSize, padding_x: f32, padding_y: f32) {
//        self.renderer.resize(size, padding_x, padding_y);
//    }
//
//    fn draw_rects(
//        self,
//        config: &Config,
//        props: &term::SizeInfo,
//        visual_bell_intensity: f64,
//        cell_line_rects: Rects
//    ) {
//        self.renderer.draw_rects(config, props, visual_bell_intensity, cell_line_rects);
//    }
//}

//impl<'a, R> RenderContext<'a> for &'a mut dyn BorrowMut<R> where
//    &'a mut R: RenderContext<'a>,
//{
//    type Renderer = &'a mut dyn Renderer;
//    type Loader = &'a mut dyn LoadGlyph;
//
//    fn with_api<'config, F, T>(
//        self,
//        config: &'config Config,
//        props: &term::SizeInfo,
//        func: F
//    ) -> T where
//        'config: 'a,
//        F: FnOnce(Self::Renderer) -> T,
//    {
//        let b = self.borrow();
//        b.with_api(config, props, |api| func(&api))
//    }
//
//    fn with_loader<F, T>(
//        self,
//        func: F
//    ) -> T where
//        F: FnOnce(Self::Loader) -> T,
//    {
//        let b = self.borrow();
//        b.with_loader(|api| func(&api))
//    }
//
//    fn resize(self, size: PhysicalSize, padding_x: f32, padding_y: f32) {
//        self.borrow().resize(size, padding_x, padding_y);
//    }
//
//    fn draw_rects(
//        self,
//        config: &Config,
//        props: &term::SizeInfo,
//        visual_bell_intensity: f64,
//        cell_line_rects: Rects
//    ) {
//        self.borrow().draw_rects(config, props, visual_bell_intensity, cell_line_rects);
//    }
//
//}

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
