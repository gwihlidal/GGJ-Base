extern crate graphics;
extern crate rand;
extern crate image;

use rand::{Rng};
use std::f32;
use piston::input::RenderArgs;
use geometry::{Point, Size};
use models::selectable::SelectableRect;
use RenderState;
use GameState;
use Assets;
use UpdateArgs;
use std_transform;
use play_pigeon_sound;
use scalar_field::*;
use std_transform_0_to_1;
use graphics::ImageSize;
use graphics::Transformed;
use graphics::*;

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

fn render_box(
    box_color: [f32; 4],
    pos: Point,
    size: Point,
    game_state: &GameState,
    render_state: &mut RenderState,
    pigeon_timer: f32,
    args: &RenderArgs,
    distress_level: f32) {

    use geometry::traits::Position;
    use gfx_graphics::*;
    use graphics::*;

    let mut verts : Vec<Point> = Vec::new();

    {
        let mut make_edge = |a: Point, b: Point| {
            let pts : i32 = 4;
            for i in 0..pts {
                let mut p : Point = a.lerp(&b, (i as f32) / (pts as f32));
                let derp = p;
                let ampl_t = ((derp.x + derp.y + pigeon_timer as f32 * 1f32) * 4f32).sin();

                let d = smoothstep(0.0f32, 0.5f32, distress_level);

                let ampl = smoothstep(0.7, 1.0, ampl_t);
                let ampl = d * (ampl + 0.6f32) * 0.007f32;

                let ampl2 = smoothstep(0.5, 1.0, ampl_t);
                let ampl2 = d * ampl2 * 0.01f32;

                p.x += (derp.y * 100f32 + pigeon_timer as f32 * 63f32).sin() as f32 * ampl;
                p.y += (derp.x * 100f32 + pigeon_timer as f32 * 71f32).sin() as f32 * ampl;

                p.x += (derp.y * 23f32 + pigeon_timer as f32 * 23f32).sin() as f32 * ampl2;
                p.y += (derp.x * 21f32 + pigeon_timer as f32 * 21f32).sin() as f32 * ampl2;

                p.x += (derp.y * 13f32 + pigeon_timer as f32 * 63f32).sin() as f32 * 0.004f32 * d;
                p.y += (derp.x *  7f32 + pigeon_timer as f32 * 51f32).sin() as f32 * 0.004f32 * d;

                verts.push(p);
            }
        };

        {
            let v0 = pos;
            let v1 = pos + Point::new(size.x, 0f32);
            let v2 = pos + Point::new(size.x, size.y);
            let v3 = pos + Point::new(0f32, size.y);
            make_edge(v0, v1);
            make_edge(v1, v2);
            make_edge(v2, v3);
            make_edge(v3, v0);
        }
    }

    let transform = std_transform();
    let mut prev = verts[verts.len() - 1];
    for i in 0..verts.len() {
        Line::new(box_color, 0.003).draw([
                prev.x as f64,
                prev.y as f64,
                verts[i].x as f64,
                verts[i].y as f64,
        ], &render_state.c.draw_state, transform, render_state.g);

        prev = verts[i];
    }
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

    pub fn update_hub(&mut self, args: &UpdateArgs) -> SystemUpdateStatus {
        let mut result = SystemUpdateStatus::AllFine;

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

            if self.distress_level > 2.0
            {
                self.destroyed = true;
                result = SystemUpdateStatus::BigBadaBoom;
            }
        }

        result
    }

    pub fn render_hubahuba(&self, render_state: &mut RenderState, args: &RenderArgs, pigeon_timer: f64) {
        use graphics::*;
        let hub = &self.hub;

        let color = [
            1f32,
            smoothstep(1f32, 0f32, self.distress_level),
            smoothstep(1f32, 0f32, self.distress_level),
            0.05f32 + 0.1f32 * smoothstep(0f32, 1f32, self.distress_level)
        ];

        let tscale = (hub.position.x.sin() * 0.2 + 1.0) as f64;
        let tscale = tscale * (1.0f64 + self.distress_level.min(1.0f32) as f64 * 10.0f64);

        let t = pigeon_timer * tscale;
        let derp = ((t * 1.235 + hub.position.x as f64).sin() * 0.5f64 + 0.5f64) as f64;
        let herp = ((t * 1.735 + hub.position.y as f64).sin() * 0.5f64 + 0.5f64) as f64;
        let rects = [
            [0.2 * ((t + hub.position.y as f64).sin() * 0.5 + 0.5) as f64, 0.0, 0.7, 1.0],
            [0.0, 0.1 + 0.1 * ((t * 1.3 + hub.position.x as f64).sin() * 0.5 + 0.5), 1.0, 0.5],
            [herp * 0.3, derp * 0.3, 0.7, 0.7],
        ];

        for &rect in rects.iter() {
            let transform = std_transform()
                .trans(hub.position.x as f64, hub.position.y as f64)
                .scale(hub.size.width as f64, hub.size.height as f64);
            graphics::rectangle(color, rect, transform, render_state.g);
        }
    }

    pub fn render_hub(&self, game_state: &GameState, render_state: &mut RenderState, args: &RenderArgs, pigeon_timer: f32) {
        let Size { width: size_x, height: size_y } = self.hub.size;
        let mut in_bounds = false;
        let aim_valid = game_state.aim_trajectory.points.len() > 0;
        if aim_valid {
            let aim_pos: Point = *game_state.aim_trajectory.points.last().unwrap();
            if self.hub.contains_point(aim_pos) {
                in_bounds = true;
            }
        }

        let tom_sucks: bool = in_bounds;
        let tom_color = if tom_sucks {
            [0.2, 0.9, 0.0, 1.0]
        } else {
            // Tom still sucks
            [1.0, 1.0, 1.0, 1.0]
        };
        render_box(tom_color, self.hub.position, Point::new(size_x, size_y), game_state, render_state, pigeon_timer, args, self.distress_level);
    }

    pub fn render_symbol(&self, assets: &Assets, render_state: &mut RenderState, args: &RenderArgs) {
        let position = self.hub.position;

        //let scale_0_to_1 = std_transform_0_to_1();
        let gui_transform = std_transform()
            //.flip_v()
           // .trans(0.0, -1.0)

            .trans(position.x as f64, position.y as f64)
             .scale(0.1 / assets.radioactive.get_width() as f64, 0.1 / assets.radioactive.get_height() as f64)
             .trans(16.0, 16.0);
        Image::new_color([1.0, 1.0, 1.0, smoothstep(0.5, 1.0, self.distress_level)]).draw(&assets.radioactive, &render_state.c.draw_state, gui_transform, render_state.g);

        /*let half_width = hub.size.width as f64 / 2.0;
        let half_height = hub.size.height as f64 / 2.0;

        let transform = std_transform()
            .trans(hub.position.x as f64 + half_width, hub.position.y as f64 + half_height)
            .scale(half_width, half_height);
        graphics::rectangle([1.0, 1.0, 0.0, 1.0], rect, transform, render_state.g);*/
    }

    pub fn reset_distress(&mut self, difficulty: f32) {
        self.distress_level_delta
            = rand::thread_rng().gen_range(DEFAULT_DISTRESS_LEVEL_DELTA * 0.5, DEFAULT_DISTRESS_LEVEL_DELTA + difficulty);
        self.distress_level = 0.0;
    }
}

#[derive(Clone)]
pub struct SystemHubCollection {
    /// All systems
    systems: Vec<SystemHub>,
    /// All connections
    connections: Vec<(SystemConnection,usize,usize)>,
    /// Obstacles
    obstacles: Vec<SelectableRect>,
    /// Rate of change, might increase over time to make it challenging
    breaking_change: f32
}

pub enum PigeonAcceptanceLevel {
    Adequate,
    GetRekd,
}

const CONNECTION_WIDTH : f32 = 0.005;
const WHITE: [f32; 4] = [1.0f32, 1.0f32, 1.0f32, 1.0f32];
const CONNECTION_THROUGHPUT: f32 = 0.05;

pub enum SystemUpdateStatus {
    AllFine,
    BigBadaBoom,
}

impl SystemHubCollection {
    /// Create a set of SystemHubs
    pub fn new() -> SystemHubCollection {

        SystemHubCollection { systems: Vec::new(), connections: Vec::new(), obstacles: Vec::new(),
                              breaking_change: 0.005 }
    }

    pub fn please_would_you_gladly_accept_a_friendly_pigeon_at_the_specified_position(&mut self, pos: Point) -> PigeonAcceptanceLevel {
        for hub in self.systems.iter_mut() {
            if !hub.destroyed && hub.hub.contains_point(pos) {
                hub.distress_level = 0.0;
                hub.reset_distress(self.breaking_change);
                return PigeonAcceptanceLevel::Adequate;
            }
        }

        PigeonAcceptanceLevel::GetRekd
    }

    pub fn pidgeon_crashing_into_wall(&self, old_style_pidgeon_pos: Point) -> bool {
        for obst in self.obstacles.iter() {
            if obst.contains_point(old_style_pidgeon_pos) {
                return true;
            }
        }
        false
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
        let pos = Point::new(0.2, 0.0);
        let size = Size::new(0.4, 0.2);
        self.systems.push(SystemHub::new(pos, size, "Reactor Chamber".to_string()));

        let pos = Point::new(0.8, 0.3);
        let size = Size::new(0.4, 0.3);
        self.systems.push(SystemHub::new(pos, size, "Kitchen".to_string()));

        let pos = Point::new(-1.4, -0.6);
        let size = Size::new(0.2, 0.5);
        self.systems.push(SystemHub::new(pos, size, "Cooling System".to_string()));

        let pos = Point::new(-1.2, 0.7);
        let size = Size::new(0.2, 0.3);
        self.systems.push(SystemHub::new(pos, size, "Command Tower".to_string()));

        for hub in self.systems.iter_mut() {
            hub.reset_distress(self.breaking_change);
        }

        // Don't flip the paramteres...
        self.add_connection(0, 1, 0, 0);
        self.add_connection(2, 0, 0, 1);
        self.add_connection(0, 3, 2, 0);
        self.add_connection(1, 3, 1, 1);
        self.add_connection(2, 1, 1, 2);

        // Add obstacles
        let pos = Point::new(-0.1, -0.3);
        let size = Size::new(0.9, 0.1);
        self.obstacles.push(SelectableRect::new(pos, size, ||{}));
    }

    pub fn update_systems(&mut self, args: &UpdateArgs) -> SystemUpdateStatus {
        let mut result = SystemUpdateStatus::AllFine;

        for mut connection in self.connections.iter_mut() {
            connection.0.borken = self.systems[connection.2].destroyed || self.systems[connection.1].destroyed;
            let factor = CONNECTION_THROUGHPUT * args.dt as f32;
            self.systems[connection.1].distress_level += self.systems[connection.2].distress_level * factor;
            self.systems[connection.2].distress_level += self.systems[connection.1].distress_level * factor;
        }

        self.breaking_change += 0.0001 * args.dt as f32; // Double as hard after ~8min
        for hub in self.systems.iter_mut() {
            hub.distress_level_delta += rand::thread_rng().gen_range(0.0, self.breaking_change * hub.distress_level_delta);
            if let SystemUpdateStatus::BigBadaBoom = hub.update_hub(args) {
                result = SystemUpdateStatus::BigBadaBoom;
        }
    }

        result
    }

    pub fn render_systems(&self, assets: &Assets, game_state: &GameState, render_state: &mut RenderState, args: &RenderArgs, pigeon_timer: f64) {
        for hub in self.systems.iter() {
            hub.render_hubahuba(render_state, args, pigeon_timer);
        }

        for hub in self.systems.iter() {
            hub.render_hub(game_state, render_state, args, pigeon_timer as f32);
        }

        for conn in self.connections.iter() {
            conn.0.render_connection(render_state, args);
        }

        for obst in self.obstacles.iter() {
            obst.render_rect(render_state, args, WHITE);
        }

        for hub in self.systems.iter() {
            hub.render_symbol(assets, render_state, args);
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

    pub fn get_game_over(&self)->bool
    {
        for hub in self.systems.iter() {
            if !hub.destroyed
            {
                return false;
            }
        }
        return true;
    }
}

#[derive(Clone)]
pub struct SystemConnection {
    /// In/middle/out point
    vertices: [Point; 3],
    /// How bad are things?
    borken: bool
}

impl SystemConnection {
    /// Create an L-shape
    pub fn new(in_port: Point, mid_port: Point, out_port: Point) -> SystemConnection {

        SystemConnection { vertices: [in_port, mid_port, out_port], borken: false}
    }

    pub fn render_connection(&self, render_state: &mut RenderState, args: &RenderArgs) {
        use graphics::*;

        let mut verts : Vec<Point> = Vec::new();

        {
            let mut make_edge = |a: Point, b: Point, bork: f32| {
                let pts = ((b - a).length() * 20.0f32 * bork).max(2.0f32) as usize;
                for i in 0..pts {
                    let lt = (i as f32) / ((pts - 1) as f32);
                    let mut p : Point = a.lerp(&b, lt);
                    let derp = p;
                    let scalar_scale = 0.5f32 + 1.5f32 * smoothstep(0.5, 0.0, (lt - 0.5f32).abs());
                    p.x += (derp.y + i as f32 * 123f32).sin() * 0.01f32 * bork * scalar_scale;
                    p.y += (derp.x + i as f32 * 324f32).sin() * 0.01f32 * bork * scalar_scale;
                    verts.push(p);
                }
            };

            let bork = if self.borken {
                1.0f32
            } else {
                0.0f32
            };

            make_edge(self.vertices[0], self.vertices[1], bork);
            make_edge(self.vertices[1], self.vertices[2], bork);
        }

        let transform = std_transform();
        for i in 1..verts.len() {
            Line::new(if self.borken { [0.7, 0.05, 0.05, 1.0] } else { WHITE }, CONNECTION_WIDTH as f64).draw([
                    verts[i-1].x as f64,
                    verts[i-1].y as f64,
                    verts[i].x as f64,
                    verts[i].y as f64,
            ], &render_state.c.draw_state, transform, render_state.g);
        }

        let rect = [(self.vertices[0].x - CONNECTION_WIDTH * 2.0) as f64, (self.vertices[0].y - CONNECTION_WIDTH * 2.0) as f64,
                    (CONNECTION_WIDTH * 4.0) as f64, (CONNECTION_WIDTH * 4.0) as f64];
        graphics::rectangle(WHITE, rect, transform, render_state.g);

        let rect = [(self.vertices[2].x - CONNECTION_WIDTH * 2.0) as f64, (self.vertices[2].y - CONNECTION_WIDTH * 2.0) as f64,
                    (CONNECTION_WIDTH * 4.0) as f64, (CONNECTION_WIDTH * 4.0) as f64];
        /*Line::new(WHITE, CONNECTION_WIDTH as f64).draw([
                    self.vertices[0].x as f64, self.vertices[0].y as f64,
                    self.vertices[1].x as f64, self.vertices[1].y as f64
            ], &render_state.c.draw_state, transform, render_state.g);
        Line::new(WHITE, CONNECTION_WIDTH as f64).draw([
                    self.vertices[1].x as f64, self.vertices[1].y as f64,
                    self.vertices[2].x as f64, self.vertices[2].y as f64
            ], &render_state.c.draw_state, transform, render_state.g);*/
    }
}