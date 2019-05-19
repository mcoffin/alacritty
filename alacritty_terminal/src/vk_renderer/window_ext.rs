use glutin::Window;
use glutin::dpi::PhysicalSize;

pub trait WindowPhysicalExt {
    fn get_inner_physical_size(&self) -> Option<PhysicalSize>;
}

impl WindowPhysicalExt for Window {
    fn get_inner_physical_size(&self) -> Option<PhysicalSize> {
        let dpi_factor = self.get_hidpi_factor();
        self.get_inner_size().map(|logical| logical.to_physical(dpi_factor))
    }
}
