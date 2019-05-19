use std::cmp;
use vulkano::swapchain;
use vulkano::swapchain::Capabilities;

pub trait CapabilitiesExt {
    fn choose_extent(&self, actual_width: u32, actual_height: u32) -> [u32; 2];
    fn choose_image_count(&self) -> u32;
}

impl CapabilitiesExt for Capabilities {
    fn choose_extent(&self, mut actual_width: u32, mut actual_height: u32) -> [u32; 2] {
        self.current_extent.unwrap_or_else(move || {
            actual_width = cmp::max(self.min_image_extent[0], cmp::min(self.max_image_extent[0], actual_width));
            actual_height = cmp::max(self.min_image_extent[1], cmp::min(self.max_image_extent[1], actual_height));
            [actual_width, actual_height]
        })
    }

    fn choose_image_count(&self) -> u32 {
        let mut image_count = self.min_image_count + 1;
        if let Some(max_image_count) = self.max_image_count {
            image_count = cmp::min(image_count, max_image_count);
        }
        image_count
    }
}

pub trait SupportedPresentModesExt {
    fn choose_present_mode(&self) -> Option<swapchain::PresentMode>;
}

impl SupportedPresentModesExt for swapchain::SupportedPresentModes {
    fn choose_present_mode(&self) -> Option<swapchain::PresentMode> {
        use swapchain::{ PresentMode, SupportedPresentModes };
        match *self {
            SupportedPresentModes { mailbox: true, .. } => Some(PresentMode::Mailbox),
            SupportedPresentModes { immediate: true, .. } => Some(PresentMode::Immediate),
            SupportedPresentModes { fifo: true, .. } => Some(PresentMode::Fifo),
            _ => None
        }
    }
}
