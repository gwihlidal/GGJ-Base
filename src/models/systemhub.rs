use geometry::{Point, Size};


#[derive(Clone)]
pub struct SystemHub {
    /// The hub's midpoint
    pub position: Point,
    /// The rectangle size
    pub size: Size,
    /// Name to be displayed
    pub name: String
}

impl SystemHub {
    /// Create a SystemHub
    pub fn new(position: Point, size: Size, name: String) -> SystemHub {
        SystemHub { position: position, size: size, name: name }
    }
}
