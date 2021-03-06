extern crate graphics;
use geometry::{Point, Size};
use RenderState;
use piston::input::RenderArgs;
use std_transform;

#[derive(Clone)]
pub struct SelectableRect {
    /// The rect's lower left corner position
    pub position: Point,
    /// The rectangle size
    pub size: Size,
    /// Function to be called on click
    pub on_click: fn(),
    /// Time on rectangle
    time_inside: f32
}

impl SelectableRect {
    /// Create a SelectableRect
    pub fn new(position: Point, size: Size, on_click: fn()) -> SelectableRect {
        SelectableRect { position: position, size: size, on_click: on_click, time_inside: 0.0 }
    }

    pub fn contains_point(&self, pt: Point) -> bool {
        self.size.contains(pt - self.position)
    }

    /// Clicked on?
    pub fn update_mouse_release(&mut self, mouse: Point) {
        if self.size.contains(mouse - self.position) {
            (self.on_click)();
        }
    }

    /// Mouse hovering?
    pub fn update_mouse_move(&mut self, dt: f32, mouse: Point) {
        if self.size.contains(mouse - self.position) {
            self.time_inside += dt;
        } else {
            self.time_inside = 0.0;
        }
    }

    /// Scale for drawing, pulsing.
    pub fn scale_factor(&self) -> f32 {
        return 1.0 + self.time_inside.cos();
    }

    pub fn upper_right_corner(&self) -> Point{
        Point::new(self.position.x + self.size.width, self.position.y + self.size.height)
    }

    pub fn render_rect(&self, render_state: &mut RenderState, args: &RenderArgs, color: [f32; 4]) {
        use graphics::*;

        let rect = [0.0, 0.0, self.size.width as f64, self.size.height as f64];

        let transform = std_transform()
            .trans(self.position.x as f64, self.position.y as f64);
        graphics::rectangle(color, rect, transform, render_state.g);
    }
}