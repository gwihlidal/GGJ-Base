extern crate graphics;
extern crate rand;
use rand::{Rng};
use std::f32;
use piston::input::RenderArgs;
use geometry::{Point, Size};
use models::selectable::SelectableRect;
use opengl_graphics;
use RenderState;
use UpdateArgs;


#[derive(Clone)]
pub struct SystemHub {
    /// The hub's rectangle
    pub hub: SelectableRect,
    /// Name to be displayed
    pub name: String,
    /// Distress level displayed, breaks at certain levels
    pub distress_level: f32,
    /// Delta of the distress level (per sec), randomly alter this one
    pub distress_level_delta: f32,
    /// Color, update by distress
    color: [f32; 4]
}

const DEFAULT_DISTRESS_LEVEL_DELTA : f32 = 0.002;

impl SystemHub {
    /// Create a SystemHub
    pub fn new(position: Point, size: Size, name: String) -> SystemHub {
        SystemHub { name: name, distress_level: 0.0, distress_level_delta: DEFAULT_DISTRESS_LEVEL_DELTA,
                    color: [1.0,0.0,1.0,1.0],
                    hub: SelectableRect::new(position, size, ||{}) } // There is an empty closure! :3
    }

    pub fn update_hub(&mut self, args: &UpdateArgs) {
        self.distress_level += self.distress_level_delta * args.dt as f32;
        self.distress_level = self.distress_level.max(0.0);

        self.color = [self.distress_level, 1.0, self.distress_level, 1.0];
        if self.distress_level > 1.0 {
            self.color = [1.0,0.0,0.0,1.0];
        }
    }

    pub fn render_hub(&self, gl: &mut opengl_graphics::GlGraphics) {
        self.hub.render_rect(gl, self.color);
    }
}

#[derive(Clone)]
pub struct SystemHubCollection {
    /// All systems
    systems: Vec<SystemHub>,
    /// Rate of change, might increase over time to make it challenging
    breaking_change: f32
}

pub enum PigeonAcceptanceLevel {
    Adequate,
    GetRekd,
}

impl SystemHubCollection {
    /// Create a set of SystemHubs
    pub fn new() -> SystemHubCollection {

        SystemHubCollection { systems: Vec::new(), breaking_change: 0.005 }
    }

    pub fn please_would_you_gladly_accept_a_friendly_pigeon_at_the_specified_position(&mut self, pos: Point) -> PigeonAcceptanceLevel {
        for hub in self.systems.iter_mut() {
            if hub.hub.contains_point(pos) {
                hub.distress_level = 0.0;
                hub.distress_level_delta = DEFAULT_DISTRESS_LEVEL_DELTA;
                return PigeonAcceptanceLevel::Adequate;
            }
        }

        PigeonAcceptanceLevel::GetRekd
    }

    pub fn init(&mut self) {
        let pos = Point::new(0.0, 0.0);
        let size = Size::new(0.4, 0.2);
        self.systems.push(SystemHub::new(pos, size, "Reactor Chamber".to_string()));

        let pos = Point::new(0.6, 0.3);
        let size = Size::new(0.4, 0.3);
        self.systems.push(SystemHub::new(pos, size, "Kitchen".to_string()));
    }

    pub fn update_systems(&mut self, args: &UpdateArgs) {
        self.breaking_change += 0.00001 * args.dt as f32; // Double as hard after ~8min
        for hub in self.systems.iter_mut() {
            hub.distress_level_delta += rand::thread_rng().gen_range(0.0, self.breaking_change * hub.distress_level_delta);
            hub.update_hub(args);
        }
    }

    pub fn render_systems(&self, gl: &mut opengl_graphics::GlGraphics) {
        for hub in self.systems.iter() {
            hub.render_hub(gl);
        }
    }
}