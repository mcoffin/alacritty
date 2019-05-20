use std::iter;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

use font::RasterizedGlyph;

use glutin::dpi::PhysicalSize;
use vulkano::device::DeviceCreationError;
use vulkano::instance::{ApplicationInfo, Instance, InstanceCreationError};
use vulkano::instance::debug::{DebugCallback, DebugCallbackCreationError};
use vulkano::swapchain::{ Surface, Swapchain, SwapchainCreationError };
use vulkano::image::swapchain::SwapchainImage;
use vulkano::sync::SharingMode;

use crate::term;
use crate::term::RenderableCell;
use crate::term::color::Rgb;
use crate::renderer::{Glyph, LoadGlyph, GlyphCache};
use crate::renderer::generic;
use crate::renderer::rects::Rects;
use crate::config::Config;
use crate::index::Line;

mod window_ext;
mod swapchain_ext;

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
                    information: true,
                    debug: true,
                };
                let cb = if enable_debug {
                    DebugCallback::new(&instance, message_types, |msg| {
                        let severity = msg.ty.log_level();
                        log!(severity, "{}: {}", msg.layer_prefix, msg.description);
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

use vulkano::device::DeviceExtensions;
use vulkano::instance::{PhysicalDevice, QueueFamily};
use vulkano::swapchain;

struct PhysicalDeviceSolution<'a> {
    physical_device: PhysicalDevice<'a>,
    graphics_family: QueueFamily<'a>,
    presentation_family: QueueFamily<'a>,
    capabilities: swapchain::Capabilities
}

const DEFAULT_QUEUE_PRIORITY: f32 = 1.0;

use vulkano::format::Format;
use vulkano::swapchain::ColorSpace;
impl<'a> PhysicalDeviceSolution<'a> {
    pub fn best_format(&self) -> Option<(Format, ColorSpace)> {
        use vulkano::format::Format::*;
        use vulkano::swapchain::ColorSpace::*;
        let exact_match = self.capabilities.supported_formats.iter()
            .find(|&(ref fmt, ref color_space)| *fmt == B8G8R8A8Unorm && *color_space == SrgbNonLinear);
        exact_match.map(|v| *v)
    }

    pub fn families(&'a self) -> Box<Iterator<Item=(QueueFamily<'a>, f32)> + 'a> {
        if self.queues_are_same() {
            Box::new(iter::once((self.graphics_family, DEFAULT_QUEUE_PRIORITY)))
        } else {
            let it = iter::once(self.graphics_family)
                .chain(iter::once(self.presentation_family))
                .map(|v| (v, DEFAULT_QUEUE_PRIORITY));
            Box::new(it)
        }
    }

    #[inline]
    fn queues_are_same(&self) -> bool {
        (self.graphics_family.id() == self.presentation_family.id())
    }

    fn image_sharing_mode(&self) -> SharingMode {
        if self.queues_are_same() {
            SharingMode::Exclusive(self.graphics_family.id())
        } else {
            SharingMode::Concurrent(vec![self.graphics_family.id(), self.presentation_family.id()])
        }
    }
}

trait PhysicalDeviceExt<'a> {
    fn supported_extensions(&self) -> DeviceExtensions;
    fn find_solution<W>(&self, surface: &Surface<W>) -> Option<PhysicalDeviceSolution<'a>>;
}

impl<'a> PhysicalDeviceExt<'a> for PhysicalDevice<'a> {
    #[inline]
    fn supported_extensions(&self) -> DeviceExtensions {
        DeviceExtensions::supported_by_device(*self)
    }

    fn find_solution<W>(&self, surface: &Surface<W>) -> Option<PhysicalDeviceSolution<'a>> {
        if !self.supported_extensions().khr_swapchain {
            return None;
        }
        self.queue_families()
            .filter(|queue_family| queue_family.queues_count() > 0)
            .find(QueueFamily::supports_graphics)
            .and_then(|graphics_family| {
                self.queue_families()
                    .find(|queue_family| surface.is_supported(queue_family.clone()).unwrap() && queue_family.queues_count() > 0)
                    .map(move |presentation_family| {
                        let capabilities = surface.capabilities(*self).unwrap();
                        PhysicalDeviceSolution {
                            physical_device: *self,
                            graphics_family: graphics_family,
                            presentation_family: presentation_family,
                            capabilities: capabilities
                        }
                    })
            })
    }
}

pub struct VkQuadRenderer<W> {
    instance: Arc<Instance>,
    surface: Arc<Surface<W>>,
    swapchain: Arc<Swapchain<W>>,
    images: Vec<Arc<SwapchainImage<W>>>,
}

pub type VulkanQuadRenderer = VkQuadRenderer<glutin::Window>;

impl VulkanQuadRenderer {
    pub fn new(instance: &VulkanInstance<glutin::Window>) -> Result<Self, VulkanInstanceCreationError> {
        let solution = PhysicalDevice::enumerate(&instance.instance)
            .filter_map(|d| d.find_solution(&instance.surface))
            .next() // TODO: choose the best solution, rather than the first
            .map(Ok)
            .unwrap_or(Err(VulkanInstanceCreationError::NoMatchingDevice))?;
        debug!("Vulkan Device: {}", solution.physical_device.name());
        let (device, mut queues) = {
            use vulkano::device::Device;
            let extensions = DeviceExtensions {
                khr_swapchain: true,
                .. DeviceExtensions::none()
            };
            let families = solution.families();
            let dev = Device::new(solution.physical_device, solution.physical_device.supported_features(), &extensions, families)
                .map_err(VulkanInstanceCreationError::DeviceCreation)?;
            dev
        };
        let (graphics_queue, presentation_queue) = if solution.queues_are_same() {
            let fst = queues.next().unwrap();
            (fst.clone(), fst)
        } else {
            queues.next()
                .and_then(move |fst| queues.next().map(move |snd| (fst, snd)))
                .unwrap()
        };
        let (mut swapchain, mut images) = {
            use vulkano::swapchain;
            use vulkano::image::ImageUsage;
            use window_ext::WindowPhysicalExt;
            use swapchain_ext::{ CapabilitiesExt, SupportedPresentModesExt };
            let image_usage = ImageUsage {
                color_attachment: true,
                .. ImageUsage::none()
            };
            let (win_width, win_height): (u32, u32) = instance.surface.window().get_inner_physical_size().unwrap().into();
            let image_sharing_mode = solution.image_sharing_mode();
            let (format, _color_space) = solution.best_format()
                .map(Ok)
                .unwrap_or(Err(VulkanInstanceCreationError::Dynamic("No suitable supported formats".into())))?;
            let present_mode = solution.capabilities.present_modes.choose_present_mode()
                .map(Ok)
                .unwrap_or(Err(VulkanInstanceCreationError::Dynamic("No suitable present modes".into())))?;
            let swap = Swapchain::new(
                device.clone(),
                instance.surface.clone(),
                solution.capabilities.choose_image_count(),
                format,
                solution.capabilities.choose_extent(win_width, win_height),
                1,
                image_usage,
                image_sharing_mode,
                swapchain::SurfaceTransform::Identity,
                swapchain::CompositeAlpha::Opaque,
                present_mode,
                true,
                None
            ).map_err(VulkanInstanceCreationError::from)?;
            swap
        };
        Ok(VkQuadRenderer {
            instance: instance.instance.clone(),
            surface: instance.surface.clone(),
            swapchain: swapchain,
            images: images
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

impl<W> generic::BaseRenderContext for VkQuadRenderer<W> {
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

impl<'a, W> generic::RenderContext<'a> for VkQuadRenderer<W> {
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
    fn log_level(&self) -> log::Level;
}

impl MessageTypesExt for vulkano::instance::debug::MessageTypes {
    fn log_level(&self) -> log::Level {
        use vulkano::instance::debug::MessageTypes;
        match self {
            &MessageTypes { error: true, .. } => log::Level::Error,
            &MessageTypes { warning: true, .. } => log::Level::Warn,
            &MessageTypes { performance_warning: true, .. } => log::Level::Warn,
            &MessageTypes { information: true, .. } => log::Level::Info,
            &MessageTypes { debug: true, .. } => log::Level::Debug,
            _ => log::Level::Trace,
        }
    }
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
    SurfaceCreation(vulkano_win::CreationError),
    NoMatchingDevice,
    DeviceCreation(DeviceCreationError),
    Dynamic(Cow<'static, str>),
    SwapchainCreation(SwapchainCreationError),
}

impl std::error::Error for VulkanInstanceCreationError {
    fn cause(&self) -> Option<&dyn (::std::error::Error)> {
        match *self {
            VulkanInstanceCreationError::Instance(ref err) => Some(err),
            VulkanInstanceCreationError::DebugCallback(ref err) => Some(err),
            VulkanInstanceCreationError::SurfaceCreation(ref err) => Some(err),
            VulkanInstanceCreationError::NoMatchingDevice => None,
            VulkanInstanceCreationError::DeviceCreation(ref err) => Some(err),
            VulkanInstanceCreationError::Dynamic(..) => None,
            VulkanInstanceCreationError::SwapchainCreation(ref err) => Some(err),
        }
    }

    fn description(&self) -> &str {
        match self {
            &VulkanInstanceCreationError::Instance(..) => "Error creating vulkan instance",
            &VulkanInstanceCreationError::DebugCallback(..) => "Error creating vulkan debug callback",
            &VulkanInstanceCreationError::SurfaceCreation(..) => "Error creating vulkan surface",
            &VulkanInstanceCreationError::NoMatchingDevice => "No matching vulkan devices!",
            &VulkanInstanceCreationError::DeviceCreation(..) => "Error creating device",
            &VulkanInstanceCreationError::Dynamic(ref description) => {
                use std::borrow::Borrow;
                description.borrow()
            },
            &VulkanInstanceCreationError::SwapchainCreation(..) => "Error creating swapchain",
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
            &VulkanInstanceCreationError::NoMatchingDevice => write!(f, "{}", description),
            &VulkanInstanceCreationError::DeviceCreation(ref err) => write!(f, "{}; {}", description, err),
            &VulkanInstanceCreationError::Dynamic(..) => write!(f, "{}", description),
            &VulkanInstanceCreationError::SwapchainCreation(ref err) => write!(f, "{}; {}", description, err),
        }
    }
}

impl From<DeviceCreationError> for VulkanInstanceCreationError {
    #[inline]
    fn from(e: DeviceCreationError) -> Self {
        VulkanInstanceCreationError::DeviceCreation(e)
    }
}

impl From<SwapchainCreationError> for VulkanInstanceCreationError {
    #[inline]
    fn from(e: SwapchainCreationError) -> Self {
        VulkanInstanceCreationError::SwapchainCreation(e)
    }
}
