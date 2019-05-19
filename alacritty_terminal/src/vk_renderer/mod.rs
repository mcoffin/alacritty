use std::iter;
use std::marker::PhantomData;
use std::sync::Arc;

use font::RasterizedGlyph;

use glutin::dpi::PhysicalSize;
use vulkano::instance::{ApplicationInfo, Instance, InstanceCreationError};
use vulkano::instance::debug::{DebugCallback, DebugCallbackCreationError};
use vulkano::swapchain::Surface;

use crate::term;
use crate::term::RenderableCell;
use crate::term::color::Rgb;
use crate::renderer::{Glyph, LoadGlyph, GlyphCache};
use crate::renderer::generic;
use crate::renderer::rects::Rects;
use crate::config::Config;
use crate::index::Line;

pub struct VulkanInstance<W> {
    instance: Arc<Instance>,
    surface: Arc<Surface<W>>,
    _debug_messenger: Option<DebugCallback>
}

const APPLICATION_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn application_version<S: AsRef<str>>(version_string: S) -> vulkano::instance::Version {
    let version_string: &str = version_string.as_ref();
    let mut number_strs = version_string.split('.');
    vulkano::instance::Version {
        major: number_strs.next().unwrap().parse().unwrap(),
        minor: number_strs.next().unwrap().parse().unwrap(),
        patch: number_strs.next().unwrap().parse().unwrap()
    }
}

impl<W> VulkanInstance<W> {
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
    pub fn surface(&self) -> &Arc<Surface<W>> {
        &self.surface
    }
}

impl VulkanInstance<glutin::Window> {
    pub fn new(enable_debug: bool, enable_verbose: bool, events_loop: &glutin::EventsLoop, window_builder: glutin::WindowBuilder) -> Result<Self, VulkanInstanceCreationError> {
        use vulkano::instance::InstanceExtensions;

        let application_info = ApplicationInfo {
            application_name: Some("alacritty".into()),
            application_version: Some(application_version(APPLICATION_VERSION)),
            engine_name: None,
            engine_version: None
        };
        let extensions = InstanceExtensions {
            ext_debug_report: enable_debug,
            .. vulkano_win::required_extensions()
        };
        let layers = if enable_debug {
            Some("VK_LAYER_LUNARG_standard_validation")
        } else {
            None
        };
        Instance::new(Some(&application_info), &extensions, layers.into_iter())
            .map_err(VulkanInstanceCreationError::Instance)
            .and_then(|instance| {
                use vulkano::instance::debug::*;
                let message_types = MessageTypes {
                    error: true,
                    warning: true,
                    performance_warning: true,
                    information: enable_verbose,
                    debug: enable_verbose,
                };
                let cb = if enable_debug {
                    DebugCallback::new(&instance, message_types, |msg| {
                        let severity = msg.ty.printing_name().unwrap_or("unknown");
                        println!("{}:{}: {}", severity, msg.layer_prefix, msg.description);
                    })
                        .map(Some)
                        .map_err(VulkanInstanceCreationError::DebugCallback)
                } else {
                    Ok(None)
                };
                cb.map(move |debug_callback| (instance, debug_callback))
            })
            .and_then(move |(instance, debug_callback)| {
                use vulkano_win::VkSurfaceBuild;
                window_builder.build_vk_surface(events_loop, instance.clone())
                    .map_err(VulkanInstanceCreationError::SurfaceCreation)
                    .map(move |surface| VulkanInstance {
                        instance: instance,
                        surface: surface,
                        _debug_messenger: debug_callback,
                    })
            })
    }
}

pub struct VulkanQuadRenderer {
    instance: Arc<VulkanInstance<glutin::Window>>,
}

impl VulkanQuadRenderer {
    pub fn new(instance: Arc<VulkanInstance<glutin::Window>>) -> Result<Self, ()> {
        Ok(VulkanQuadRenderer {
            instance: instance.clone()
        })
    }
}

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

trait MessageTypesExt {
    fn printing_name(&self) -> Option<&'static str>;
}

impl MessageTypesExt for vulkano::instance::debug::MessageTypes {
    fn printing_name(&self) -> Option<&'static str> {
        use vulkano::instance::debug::MessageTypes;
        match self {
            &MessageTypes { error: true, .. } => Some("error"),
            &MessageTypes { warning: true, .. } => Some("warning"),
            &MessageTypes { performance_warning: true, .. } => Some("performance_warning"),
            &MessageTypes { information: true, .. } => Some("information"),
            &MessageTypes { debug: true, .. } => Some("debug"),
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum VulkanInstanceCreationError {
    Instance(InstanceCreationError),
    DebugCallback(DebugCallbackCreationError),
    SurfaceCreation(vulkano_win::CreationError)
}

impl std::error::Error for VulkanInstanceCreationError {
    fn cause(&self) -> Option<&dyn (::std::error::Error)> {
        match *self {
            VulkanInstanceCreationError::Instance(ref err) => Some(err),
            VulkanInstanceCreationError::DebugCallback(ref err) => Some(err),
            VulkanInstanceCreationError::SurfaceCreation(ref err) => Some(err),
        }
    }

    fn description(&self) -> &str {
        match self {
            &VulkanInstanceCreationError::Instance(..) => "Error creating vulkan instance",
            &VulkanInstanceCreationError::DebugCallback(..) => "Error creating vulkan debug callback",
            &VulkanInstanceCreationError::SurfaceCreation(..) => "Error creating vulkan surface",
        }
    }
}

impl std::fmt::Display for VulkanInstanceCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;
        let description = self.description();
        match self {
            &VulkanInstanceCreationError::Instance(ref err) => write!(f, "{}; {}", description, err),
            &VulkanInstanceCreationError::DebugCallback(ref err) => write!(f, "{}; {}", description, err),
            &VulkanInstanceCreationError::SurfaceCreation(ref err) => write!(f, "{}; {}", description, err),
        }
    }
}
