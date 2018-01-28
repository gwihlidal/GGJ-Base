extern crate graphics;
extern crate rand;
use rand::{Rng};
use std::f32;
use piston::input::RenderArgs;
use geometry::{Point, Size};
use models::selectable::SelectableRect;
use RenderState;
use UpdateArgs;
use std_transform;


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
    color: [f32; 4],
    // Flag for if the square has suffered a meltdown or not
    pub destroyed: bool,
    //Poisition of the center of the square
    pub center: Point
}

const DEFAULT_DISTRESS_LEVEL_DELTA : f32 = 0.002;

impl SystemHub {
    /// Create a SystemHub
    pub fn new(position: Point, size: Size, name: String) -> SystemHub {
        SystemHub { name: name, distress_level: 0.0, distress_level_delta: DEFAULT_DISTRESS_LEVEL_DELTA,
                    color: [1.0,0.0,1.0,1.0],
                    destroyed : false,
                    center : Point::new(position.x+(size.width/2.0),position.y+(size.height/2.0)),
                    hub: SelectableRect::new(position, size, ||{}) } // There is an empty closure! :3
    }

    pub fn update_hub(&mut self, args: &UpdateArgs) {
        if self.destroyed
        {
            self.color = [0.0,0.0,0.0,1.0];
        }
        else {
            self.distress_level += self.distress_level_delta * args.dt as f32;
            self.distress_level = self.distress_level.max(0.0);

            self.color = [self.distress_level, 1.0, self.distress_level, 1.0];
            if self.distress_level > 1.0 {
                self.color = [1.0,0.0,0.0,1.0];
            }

            if self.distress_level > 3.0
            {
                self.destroyed = true;
            }
        }
    }

    pub fn render_hub(&self, render_state: &mut RenderState, args: &RenderArgs) {
        self.hub.render_rect(render_state, args, self.color);
    }
}

#[derive(Clone)]
pub struct SystemHubCollection {
    /// All systems
    systems: Vec<SystemHub>,
    /// All connections
    connections: Vec<(SystemConnection,usize,usize)>,
    /// Rate of change, might increase over time to make it challenging
    breaking_change: f32
}

pub enum PigeonAcceptanceLevel {
    Adequate,
    GetRekd,
}

const CONNECTION_WIDTH : f32 = 0.005;

impl SystemHubCollection {
    /// Create a set of SystemHubs
    pub fn new() -> SystemHubCollection {

        SystemHubCollection { systems: Vec::new(), connections: Vec::new(), breaking_change: 0.005 }
    }

    pub fn please_would_you_gladly_accept_a_friendly_pigeon_at_the_specified_position(&mut self, pos: Point) -> PigeonAcceptanceLevel {
        for hub in self.systems.iter_mut() {
            if hub.destroyed{
                return PigeonAcceptanceLevel::GetRekd;
            }
            if hub.hub.contains_point(pos) {
                hub.distress_level = 0.0;
                hub.distress_level_delta = DEFAULT_DISTRESS_LEVEL_DELTA;
                return PigeonAcceptanceLevel::Adequate;
            }
        }

        PigeonAcceptanceLevel::GetRekd
    }

    fn add_connection(&mut self, a_idx: usize, b_idx: usize, num_conns_a: i32, num_conns_b: i32) {
        let a = &self.systems[a_idx];
        let b = &self.systems[b_idx];
        let mut dist_right_up  = b.hub.position - a.hub.upper_right_corner();
        let mut dist_left_down = a.hub.position - b.hub.upper_right_corner();
        
        let mut vertices: [Point; 3] = [Point::new(0.0,0.0), Point::new(0.0,0.0), Point::new(0.0,0.0)];


        let corner_pos = [if dist_right_up.x.abs() < dist_left_down.x.abs() {1.0} else {0.0},
                          if dist_right_up.y.abs() < dist_left_down.y.abs() {1.0} else {0.0}];
        let corner_offset = [corner_pos[0] * 0.5 + 0.5, corner_pos[1] * 0.5 + 0.5];

        vertices[0].x = a.hub.position.x + corner_pos[0] * a.hub.size.width;
        vertices[2].x = b.hub.position.x + (1.0 - corner_pos[0]) * b.hub.size.width;

        vertices[0].y = a.hub.position.y + corner_pos[1] * a.hub.size.height;
        vertices[2].y = b.hub.position.y + (1.0 - corner_pos[1]) * b.hub.size.height;

        let dir = [vertices[2].x - vertices[0].x, vertices[2].y - vertices[2].y];
        let dir_sign = [dir[0].signum(), dir[1].signum()];

        // From hub a, go in x or y direction?
        if dir[0] > dir[1] {
            // Go in x
            // Space out connections
            vertices[0].y += CONNECTION_WIDTH * 6.0 * -corner_offset[1] * dir_sign[1] * (num_conns_a as f32 * 2.0 + 1.0);
            vertices[2].x += CONNECTION_WIDTH * 6.0 *  corner_offset[0] * dir_sign[0] * (num_conns_b as f32 * 2.0 + 1.0);

            vertices[1].x = vertices[2].x;
            vertices[1].y = vertices[0].y;
        }
        else {
            // Go in y
            // Space out connections
            vertices[0].x += CONNECTION_WIDTH * 6.0 * -corner_offset[0] * dir_sign[0] * (num_conns_a as f32 * 2.0 + 1.0);
            vertices[2].y += CONNECTION_WIDTH * 6.0 *  corner_offset[1] * dir_sign[1] * (num_conns_b as f32 * 2.0 + 1.0);

            vertices[1].x = vertices[0].x;
            vertices[1].y = vertices[2].y;
        }

        self.connections.push((SystemConnection::new(vertices[0], vertices[1], vertices[2]), a_idx, b_idx));
    }

    pub fn init(&mut self) {
        let pos = Point::new(0.0, 0.0);
        let size = Size::new(0.4, 0.2);
        self.systems.push(SystemHub::new(pos, size, "Reactor Chamber".to_string()));

        let pos = Point::new(0.6, 0.3);
        let size = Size::new(0.4, 0.3);
        self.systems.push(SystemHub::new(pos, size, "Kitchen".to_string()));

        let pos = Point::new(-1.4, -0.6);
        let size = Size::new(0.2, 0.5);
        self.systems.push(SystemHub::new(pos, size, "Cooling System".to_string()));

        let pos = Point::new(-1.2, 0.7);
        let size = Size::new(0.2, 0.3);
        self.systems.push(SystemHub::new(pos, size, "Command Tower".to_string()));

        // Don't flip the paramteres...
        self.add_connection(0, 1, 0, 0);
        self.add_connection(2, 0, 0, 1);
        self.add_connection(0, 3, 2, 0);
        self.add_connection(1, 3, 1, 1);
        self.add_connection(2, 1, 1, 2);
    }

    pub fn update_systems(&mut self, args: &UpdateArgs) {
        self.breaking_change += 0.0001 * args.dt as f32; // Double as hard after ~8min
        for hub in self.systems.iter_mut() {
            hub.distress_level_delta += rand::thread_rng().gen_range(0.0, self.breaking_change * hub.distress_level_delta);
            hub.update_hub(args);
        }
    }

    pub fn render_systems(&self, render_state: &mut RenderState, args: &RenderArgs) {
        for hub in self.systems.iter() {
            hub.render_hub(render_state, args);
        }

        for conn in self.connections.iter() {
            conn.0.render_connection(render_state, args);
        }
    }

    pub fn get_pos(& self) -> Vec<Point>
    {
        let mut positions = Vec::new();

        for hub in self.systems.iter() {
            positions.push(hub.center);
        }

        return positions;
    }

    pub fn get_destroyed(& self) -> Vec<bool>
    {
        let mut positions = Vec::new();
        for hub in self.systems.iter() {
            positions.push(hub.destroyed);
        }

        return positions;
    }
}

#[derive(Clone)]
pub struct SystemConnection {
    /// In/middle/out point
    vertices: [Point; 3],
    /// How bad are things?
    wiggle_level: f32
}

impl SystemConnection {
    /// Create an L-shape
    pub fn new(in_port: Point, mid_port: Point, out_port: Point) -> SystemConnection {

        SystemConnection { vertices: [in_port, mid_port, out_port], wiggle_level: 0.0 }
    }

    pub fn render_connection(&self, render_state: &mut RenderState, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0f32, 1.0f32, 1.0f32, 1.0f32];
        let transform = std_transform();

        Line::new(WHITE, CONNECTION_WIDTH as f64).draw([
                    self.vertices[0].x as f64, self.vertices[0].y as f64,
                    self.vertices[1].x as f64, self.vertices[1].y as f64
            ], &render_state.c.draw_state, transform, render_state.g);
        Line::new(WHITE, CONNECTION_WIDTH as f64).draw([
                    self.vertices[1].x as f64, self.vertices[1].y as f64,
                    self.vertices[2].x as f64, self.vertices[2].y as f64
            ], &render_state.c.draw_state, transform, render_state.g);

        for vertex in self.vertices.iter() {
            let rect = [(vertex.x - CONNECTION_WIDTH * 2.0) as f64, (vertex.y - CONNECTION_WIDTH * 2.0) as f64,
                        (CONNECTION_WIDTH * 4.0) as f64, (CONNECTION_WIDTH * 4.0) as f64];
            graphics::rectangle(WHITE, rect, transform, render_state.g);
        }
    }
}