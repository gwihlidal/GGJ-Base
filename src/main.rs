#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

extern crate glutin_window;
extern crate piston;
extern crate graphics;
extern crate gfx_graphics;
extern crate find_folder;
#[macro_use] extern crate gfx;
extern crate gfx_device_gl;
extern crate rodio;
extern crate rand;
extern crate image;

use rand::Rng;
use std::path::Path;
use std::io::BufReader;
use std::f32;
use std::f64;

use gfx::traits::*;
use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use glutin_window::*;
use piston::window::{OpenGLWindow, Window, WindowSettings};
use piston::event_loop::{Events, EventSettings, EventLoop};
use graphics::draw_state::Blend;
use graphics::*;
use piston::input::*;
use gfx_graphics::*;
use gfx::{Device, VertexBuffer, RenderTarget};
use gfx::traits::FactoryExt;

mod scalar_field;
use scalar_field::*;

#[macro_use]
pub mod geometry;
use geometry::point::{Point};
use geometry::size::Size;

#[macro_use]
pub mod models;

#[allow(unused_imports)]
use graphics::math::Matrix2d;

use models::pigeon::*;
use models::coop::Coop;
use models::speechbubble::SpeechBubble;
use models::systemhub::*;
use geometry::traits::Collide;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_pos",
        uv: [f32; 2] = "a_uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_color",
        out: gfx::RenderTarget<Srgba8> = "Target0",
    }
}

const QUAD_VERTS: [Vertex; 6] = [
	Vertex { pos: [-1.0, -1.0], uv: [0.0, 0.0] }, // Bottom Left
	Vertex { pos: [ 1.0, -1.0], uv: [1.0, 0.0] }, // Bottom Right
	Vertex { pos: [-1.0,  1.0], uv: [0.0, 1.0] }, // Top Left
    Vertex { pos: [-1.0,  1.0], uv: [0.0, 1.0] }, // Top Left
	Vertex { pos: [ 1.0, -1.0], uv: [1.0, 0.0] }, // Bottom Right
	Vertex { pos: [ 1.0,  1.0], uv: [1.0, 1.0] }, // Top Right
];

pub struct RenderState<'a, 'b: 'a> {
    g: &'a mut GfxGraphics<'b, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    c: graphics::Context,
}

pub struct GameState {
    pigeons: Vec<Pigeon>,
    coops: Vec<Coop>,
    bubbles: Vec<SpeechBubble>,
    system_hubs: SystemHubCollection,
    irradiance_field: ScalarField,
    aim_trajectory: Trajectory,
    selected_coop: Option<usize>,
    game_over: bool,
    game_over_time: f64,
    starting: bool,
    pigeon_timer: f64,
}

pub struct Assets {
    radioactive: Texture<gfx_device_gl::Resources>,
    starting: Texture<gfx_device_gl::Resources>,
    game_over: Texture<gfx_device_gl::Resources>,
    pigeon_points_f0: Vec<(geometry::Point, geometry::Point)>,
    pigeon_points_f1: Vec<(geometry::Point, geometry::Point)>,
}

fn pos_to_irradiance_coord(p: Point) -> Point {
	let aspect = 16.0 / 9.0;
	(p + Point::new(aspect, 1.0)) / Point::new(aspect * 2.0, 2.0)
}

fn simulate_trajectory(game_state: &GameState, origin: Point, cursor: Point) -> Trajectory {
    	let mut pos = origin;
    	let mut vel = (cursor - pos) * 0.5f32;
    	//let mut vel = vel * 0.1f32;

    	let mut points = Vec::new();

    	let iter_count = 200;
    	let delta_t = 0.07f32;

    	for _ in 0..iter_count {
    		points.push(pos);
        let grad = game_state.irradiance_field.sample_gradient(pos_to_irradiance_coord(pos));
    		vel = vel * 0.98 + grad * 0.13;

    		pos = pos + vel * delta_t;

    		// TODO: x bounds
    		if pos.y > 1f32 || pos.y < -1f32 {
    			break;
    		}
    	}

    	Trajectory { points }
    }

static mut SHAKE_IT: Point = Point { x: 0.0, y: 0.0 };
static mut SHAKE_ON: bool = false;
static mut SHAKE_RADIUS: f32 = 0.1;
static mut SHAKE_ANGLE: f32 = 0.0;

fn std_transform() -> Matrix2d {
    use graphics::*;
    let aspect = 16.0 / 9.0;
    let viewport_center = Point { x:0.0, y:0.0 };
    unsafe {
        if SHAKE_ON {
            graphics::math::identity().scale(1.0 / aspect, 1.0).trans(SHAKE_IT.x as f64, SHAKE_IT.y as f64)
        } else {
            graphics::math::identity().scale(1.0 / aspect, 1.0)
        }
    }
}

fn play_camera_shake() {
    unsafe {
        SHAKE_RADIUS = 0.15;
        SHAKE_ANGLE = rand::thread_rng().gen_range(1.0, 360.0);
        SHAKE_IT = Point { x: SHAKE_ANGLE.sin() * SHAKE_RADIUS, y: SHAKE_ANGLE.cos() * SHAKE_RADIUS };
        SHAKE_ON = true;
    }
}

fn on_mouse_move(game_state: &mut GameState, mouse: [f64;2]) {
    let mut coop = &mut game_state.coops[0];
    Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));

    for bubble in game_state.bubbles.iter_mut() {
        SpeechBubble::update_mouse_move(bubble, 1.0, Point::new(mouse[0] as f32, mouse[1] as f32));
    }
}

fn on_mouse_click(game_state: &mut GameState, mouse: [f64;2]) {
		let coop_idx = 0;
    	let mut coop = &mut game_state.coops[coop_idx];
    	let fake_click = coop.position;
        if Coop::update_mouse_click(coop, fake_click) {
        	game_state.selected_coop = Some(coop_idx);
        }

        // lololo
        Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));
}

fn on_mouse_release(game_state: &mut GameState, mouse: [f64;2]) {
    let coop = &mut game_state.coops[0];
    if let Some(mut pigeon) = Coop::update_mouse_release(coop) {
        pigeon.trajectory = Some(game_state.aim_trajectory.clone());
        game_state.pigeons.push(pigeon);
        play_pigeon_sound();
    }

    for mut bubble in game_state.bubbles.iter_mut() {
        SpeechBubble::update_mouse_release(bubble, Point::new(mouse[0] as f32, mouse[1] as f32));
    }

    game_state.selected_coop = None;
}

fn on_load(assets: &mut Assets, game_state: &mut GameState) {
    let pos_coop = geometry::Point::new(0.0, -0.9);
    game_state.coops.push(Coop::new(pos_coop));
    game_state.system_hubs.init();

    //let pos_bubble = geometry::Point::new(0.2, 0.5);
    //game_state.bubbles.push(SpeechBubble::new(pos_bubble,Size::new(0.4, 0.1), play_pigeon_sound, pos_bubble));

    // Pigeon animation frame #0
    assets.pigeon_points_f0.push((Point::new(400.0, 442.043),   Point::new(100.0, 442.043)));
    assets.pigeon_points_f0.push((Point::new(100.0, 442.043),   Point::new(250.443, 57.113)));
    assets.pigeon_points_f0.push((Point::new(250.443, 57.113),  Point::new(400.0, 442.043)));
    assets.pigeon_points_f0.push((Point::new(309.156, 205.907), Point::new(445.678, 147.404)));
    assets.pigeon_points_f0.push((Point::new(445.678, 147.404), Point::new(375.655, 376.547)));
    assets.pigeon_points_f0.push((Point::new(375.655, 376.547), Point::new(309.156, 205.907)));
    assets.pigeon_points_f0.push((Point::new(191.678, 205.907), Point::new(55.156, 147.404)));
    assets.pigeon_points_f0.push((Point::new(55.156, 147.404),  Point::new(125.179, 376.547)));
    assets.pigeon_points_f0.push((Point::new(125.179, 376.547), Point::new(191.678, 205.907)));

    // Pigeon animation frame #1
    assets.pigeon_points_f1.push((Point::new(400.0, 442.043),   Point::new(100.0, 442.043)));
    assets.pigeon_points_f1.push((Point::new(100.0, 442.043),   Point::new(250.443, 57.113)));
    assets.pigeon_points_f1.push((Point::new(250.443, 57.113),  Point::new(400.0, 442.043)));
    assets.pigeon_points_f1.push((Point::new(308.156, 205.907), Point::new(449.411, 311.876)));
    assets.pigeon_points_f1.push((Point::new(449.411, 311.876), Point::new(374.655, 376.547)));
    assets.pigeon_points_f1.push((Point::new(374.655, 376.547), Point::new(308.156, 205.907)));
    assets.pigeon_points_f1.push((Point::new(192.411, 205.907), Point::new(51.156, 311.876)));
    assets.pigeon_points_f1.push((Point::new(51.156, 311.876),  Point::new(125.912, 376.547)));
    assets.pigeon_points_f1.push((Point::new(125.912, 376.547), Point::new(192.411, 205.907)));
}

fn on_update(game_state: &mut GameState, args: &UpdateArgs, cursor: Point) {
	if game_state.starting {
		return;
	}

    // Radioactive decay
    game_state.irradiance_field.decay(0.88f32.powf(args.dt as f32));

    // Fixed radiation source for the reactor or whatever
    game_state.irradiance_field.splat(pos_to_irradiance_coord(Point::new(-0.5f32, 0.5f32)), 4f32, RadiationBlendMode::Max);

    let mut pigeon_to_nuke = None;
    for i in 0..game_state.pigeons.len() {
    let mut pigeon = &mut game_state.pigeons[i];
        if PigeonStatus::ReachedDestination == pigeon.update((1.0 * args.dt) as f32)
           || game_state.system_hubs.pidgeon_crashing_into_wall(pigeon.vector.position) {
            pigeon_to_nuke = Some(i);
        }
    }

    for coop in game_state.coops.iter_mut() {
        coop.update(args.dt as f32);
    }

    if let Some(i) = pigeon_to_nuke {
        let pos = game_state.pigeons[i].vector.position;
        game_state.pigeons.swap_remove(i);

        if let PigeonAcceptanceLevel::GetRekd =
            game_state.system_hubs.please_would_you_gladly_accept_a_friendly_pigeon_at_the_specified_position(pos) {
            game_state.irradiance_field.splat(pos_to_irradiance_coord(pos), 4f32, RadiationBlendMode::Add);
        }
    }

    if let Some(coop_idx) = game_state.selected_coop {
        game_state.aim_trajectory =
            simulate_trajectory(&game_state, game_state.coops[coop_idx].position, cursor);
    }

    // Irradiate destroyed structures
    let mut positions: Vec<Point> =  game_state.system_hubs.get_pos();
    let mut destroyed: Vec<bool> =  game_state.system_hubs.get_destroyed();
    let len : usize = positions.len();

    for i in 0..len {
        if destroyed[i]
        {
         game_state.irradiance_field.splat(pos_to_irradiance_coord(positions[i]), 7f32, RadiationBlendMode::Max);
        }
    }

    // If all structures are destroyed, GAME OVER!
    if game_state.system_hubs.get_game_over() && !game_state.game_over
    {
        game_state.game_over = true;
        game_state.game_over_time = game_state.pigeon_timer;
    }

    game_state.pigeon_timer += args.dt;
    if let SystemUpdateStatus::BigBadaBoom = game_state.system_hubs.update_systems(args) {
    	play_camera_shake();
    }

    unsafe {
        if SHAKE_ON
        {
            if SHAKE_RADIUS <= 0.001 {
                SHAKE_ON = false;
            }

            SHAKE_ANGLE += 180.0 - rand::thread_rng().gen_range(1.0, 60.0);
            SHAKE_RADIUS *= 0.95;
            SHAKE_IT = Point { x: SHAKE_ANGLE.sin() * SHAKE_RADIUS, y: SHAKE_ANGLE.cos() * SHAKE_RADIUS };
        }
    }
}

#[allow(deprecated)]
fn play_sound(sound_file: &str) {
    #[allow(deprecated)]
    let endpoint = rodio::get_default_endpoint().unwrap();
    let sink = rodio::Sink::new(&endpoint);

    let file = std::fs::File::open(sound_file).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    sink.append(source);
    sink.detach();
}

fn play_pigeon_sound()
{
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x: u32 = rng.gen_range(1,12);
    let s: String = x.to_string();
    let ss: &str = &s;

    let mut sound_file = String::from("assets/coo");
    sound_file.push_str(ss);
    sound_file.push_str(".wav");
    play_sound(&sound_file);
}

fn std_transform_0_to_1() -> Matrix2d {
    use graphics::*;

    let aspect = 16.0 / 9.0;
    std_transform()
    	.scale(aspect, 1.0)
    	.trans(-1.0, -1.0).scale(2.0, 2.0)
}

fn render_irradiance(
    factory: &mut gfx_device_gl::Factory,
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    if game_state.starting || game_state.game_over {
        return;
    }

    let sf = &game_state.irradiance_field;
    let sf_texture = Texture::from_image(
        factory,
        &sf.to_image_buffer(),
        &TextureSettings::new()
    ).unwrap();

    let scale_0_to_1 = std_transform_0_to_1();
    Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(
        &sf_texture,
        &Default::default(),
        scale_0_to_1.scale(1.0 / sf.width as f64, 1.0 / sf.height as f64),
        render_state.g
    );
}

fn render_trajectory(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    if game_state.starting || game_state.game_over {
        return;
    }

    use graphics::*;
    if let Some(coop) = game_state.selected_coop {
        let trajectory = &game_state.aim_trajectory;
        if trajectory.points.len() < 2 {
            return;
        }

        let col = if game_state.coops[coop].can_fire() {
            let t = ((game_state.pigeon_timer * 30.0).sin() * 0.3 + 0.7) as f32;
            [0.831 * t, 0.812 * t, 0.416 * t, 1.0]
        } else {
            let t = ((game_state.pigeon_timer * 10.0).sin() * 0.3 + 0.7) as f32;
            [0.408 * t, 0.067 * t, 0.247 * t, 0.5]
        };

        for i in 1..trajectory.points.len() {
            Line::new(col, 0.005).draw([
                trajectory.points[i-1].x as f64,
                trajectory.points[i-1].y as f64,
                trajectory.points[i].x as f64,
                trajectory.points[i].y as f64,
            ], &Default::default(), std_transform(), render_state.g);
	    }
    }
}

fn render_pigeons(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    if game_state.starting || game_state.game_over {
        return;
    }

    use geometry::traits::Position;

    const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    for pigeon in game_state.pigeons.iter() {
        let square = graphics::rectangle::square(0.0, 0.0, 0.1);
        let rotation = pigeon.vector.direction as f64;

        let transform = std_transform()
				.trans(pigeon.x() as f64, pigeon.y() as f64)
				.rot_rad(rotation);

        let remapped = (game_state.pigeon_timer * 40.0).sin() * 0.5 + 0.5; // -1...1 -> 0...1

        let scale = 0.00023;
        for i in 0..assets.pigeon_points_f0.len() {
            let (s_o, e_o) = assets.pigeon_points_f0[i];
            let (s_p, e_p) = assets.pigeon_points_f1[i];
            let animated_s = s_o.lerp(&s_p, remapped as f32);
            let animated_e = e_o.lerp(&e_p, remapped as f32);
	        Line::new([1.0f32, 1.0f32, 1.0f32, 1.0f32], 0.002).draw([
	                (animated_s.x - 256.0) as f64 * scale,
	                (animated_s.y - 256.0) as f64 * -scale,
	                (animated_e.x - 256.0) as f64 * scale,
	                (animated_e.y - 256.0) as f64 * -scale,
	        ], &render_state.c.draw_state, transform, render_state.g);
	    }
    }
}

const MARCHING_SQUARES_SOLID: &'static [&'static [&'static [[f64; 2]]]] = &[
	&[&[]],	// 0000
	&[&[ [0f64, 0f64], [1f64, 0f64], [0f64, 1f64] ]],	// 0001
	&[&[ [1f64, 0f64], [2f64, 0f64], [2f64, 1f64] ]],	// 0010
	&[&[ [0f64, 0f64], [2f64, 0f64], [2f64, 1f64], [0f64, 1f64]]],	// 0011
	&[&[ [2f64, 1f64], [2f64, 2f64], [1f64, 2f64] ]],	// 0100
	&[&[ [2f64, 1f64], [2f64, 2f64], [1f64, 2f64] ], &[ [0f64, 0f64], [1f64, 0f64], [0f64, 1f64] ]],	// 0101
	&[&[ [1f64, 0f64], [2f64, 0f64], [2f64, 2f64], [1f64, 2f64] ]],	// 0110
	&[&[ [0f64, 0f64], [2f64, 0f64], [2f64, 2f64], [1f64, 2f64], [0f64, 1f64] ]],	// 0111
	&[&[ [0f64, 1f64], [1f64, 2f64], [0f64, 2f64] ]],	// 1000
	&[&[ [0f64, 0f64], [1f64, 0f64], [1f64, 2f64], [0f64, 2f64] ]],	// 1001
	&[&[ [0f64, 1f64], [1f64, 2f64], [0f64, 2f64] ], &[ [1f64, 0f64], [2f64, 0f64], [2f64, 1f64] ]],	// 1010
	&[&[ [0f64, 0f64], [2f64, 0f64], [2f64, 1f64], [1f64, 2f64], [0f64, 2f64] ]],	// 1011
	&[&[ [0f64, 1f64], [2f64, 1f64], [2f64, 2f64], [0f64, 2f64] ]],	// 1100
	&[&[ [0f64, 0f64], [1f64, 0f64], [2f64, 1f64], [2f64, 2f64], [0f64, 2f64] ]],	// 1101
	&[&[ [1f64, 0f64], [2f64, 0f64], [2f64, 2f64], [0f64, 2f64], [0f64, 1f64] ]],	// 1110
	&[&[ [0f64, 0f64], [2f64, 0f64], [2f64, 2f64], [0f64, 2f64] ]],	// 1111
];

fn render_radiation(game_state: &GameState, render_state: &mut RenderState, sf: &ScalarField, time: f64) {
	use graphics::*;

    if game_state.starting || game_state.game_over {
        return;
    }

	const X_COUNT : usize = 16 * 2;
	const Y_COUNT : usize = 9 * 2;

    let transform = std_transform_0_to_1()
        .scale(1.0 / X_COUNT as f64, 1.0 / Y_COUNT as f64);

	const X_BUF : usize = X_COUNT + 1;
	const Y_BUF : usize = Y_COUNT + 1;
    let mut h_samples = [0f32; X_BUF * Y_BUF];

	for y in 0..Y_BUF {
		let y0 = y as f32 / Y_COUNT as f32;
    	for x in 0..X_BUF {
    		let x0 = x as f32 / X_COUNT as f32;
    		h_samples[X_BUF * y + x] = sf.sample(Point::new(x0, y0));
    	}
    }

    let mut draw = |th, col| {
    	for y in 0..Y_COUNT {
        	for x in 0..X_COUNT {
        		let h0 = h_samples[(y+0) * X_BUF + (x+0)];
        		let h1 = h_samples[(y+0) * X_BUF + (x+1)];
        		let h2 = h_samples[(y+1) * X_BUF + (x+1)];
        		let h3 = h_samples[(y+1) * X_BUF + (x+0)];

        		let h0 = if h0 > th { 1 } else { 0 };
        		let h1 = if h1 > th { 1 } else { 0 };
        		let h2 = if h2 > th { 1 } else { 0 };
        		let h3 = if h3 > th { 1 } else { 0 };

        		let polys = MARCHING_SQUARES_SOLID[h0 + 2 * h1 + 4 * h2 + 8 * h3];
        		for poly in polys.iter() {
		            let transform = transform.trans(x as f64, y as f64).scale(0.5, 0.5);
        			//graphics::polygon(col, &poly[..], transform, gl);

        			let mut p : Vec<[f64; 2]> = Vec::new();
        			for v in poly.iter() {
        				p.push(*v);
        			}

        			for i in 0..p.len() {
        				let x_t = p[i][1] * 0.5 + y as f64 + time * 2.1234;
        				let y_t = p[i][0] * 0.5 + x as f64 + time * 1.865;
        				p[i][0] += x_t.sin() * 0.25;
        				p[i][1] += y_t.sin() * 0.25;
        			}
        			//graphics::polygon(col, &p[..], transform, gl);
        			Polygon::new(col).draw(&p[..], &render_state.c.draw_state, transform, render_state.g);
        		}
        	}
       	}
    };

    let col = [0.635, 0.773, 0.388, 0.1];
    draw(0.3f32, col);
    draw(0.6f32, col);
    //draw(0.8f32, col);
}

fn render_coop(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    if game_state.game_over || game_state.starting {
        return;
    }

    for coop in game_state.coops.iter() {
        use graphics::*;
        use geometry::traits::Position;

        const COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        use graphics::math::Vec2d;

        let mut transform = std_transform()
            .trans(coop.x() as f64, coop.y() as f64)
            .scale(coop.radius() as f64, coop.radius() as f64);

            let mut pentagon: [Vec2d;5] = [[1.0,0.0], [0.309, -0.951], [-0.809, -0.588], [-0.809, 0.588], [0.309, 0.951]];
            if let Some(dir) = coop.direction {
                pentagon[0][0] *= 2.0;
                transform = transform.rot_rad(dir as f64);
            }

        Polygon::new(COLOR).draw(&pentagon, &render_state.c.draw_state, transform, render_state.g);
    }
}

fn render_hubs(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    if game_state.starting || game_state.game_over {
        return;
    }

    game_state.system_hubs.render_systems(assets, game_state, render_state, args, game_state.pigeon_timer);
    for bubble in game_state.bubbles.iter() {
            bubble.render_bubble(render_state, args);
    }

    //Speech Bubble Text
    /*for bubble in game_state.bubbles.iter() {
                let s = bubble.get_text();
                let ss: &str = &s;
                let position: &Point = bubble.get_point();
        let h_offset: f32 = bubble.get_height()/2.0;

                text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw(ss,
                                                             glyph_cache,
            &render_state.c.draw_state,
            std_transform()
                                                                 .flip_v()
                .trans(position.x as f64, -position.y as f64 - h_offset as f64 + 0.01 as f64)
                .scale(0.001  as f64,0.001 as f64),
            render_state.g).unwrap();
    }*/
}

fn render_ui(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {
        if game_state.game_over {
            let scale_0_to_1 = std_transform_0_to_1();
                    let gui_transform = scale_0_to_1
                        .flip_v()
                        .trans(0.0, -1.0)
                        .scale(1.0 / assets.game_over.get_width() as f64, 1.0 / assets.game_over.get_height() as f64);
            Image::new().draw(&assets.game_over, &render_state.c.draw_state, gui_transform, render_state.g);
        } else if game_state.starting {
            let scale_0_to_1 = std_transform_0_to_1();
                    let gui_transform = scale_0_to_1
                        .flip_v()
                        .trans(0.0, -1.0)
                        .scale(1.0 / assets.starting.get_width() as f64, 1.0 / assets.starting.get_height() as f64);
            Image::new().draw(&assets.starting, &render_state.c.draw_state, gui_transform, render_state.g);
        }
    }

fn main() {

    let opengl = OpenGL::V3_2;
    let samples = 1;
    let mut window: GlutinWindow = WindowSettings::new(
            "Irradiant Descent",
            [1920, 1080]
        )
        .exit_on_esc(true)
        .samples(samples)
        .opengl(opengl)
        .build()
        .unwrap();

    let (mut device, mut factory) = gfx_device_gl::create(|s|
        window.get_proc_address(s) as *const std::os::raw::c_void);

    let draw_size = window.draw_size();
    let aa = samples as gfx::texture::NumSamples;
    let dim = (draw_size.width as u16, draw_size.height as u16, 1, aa.into());
    let color_format = <Srgba8 as Formatted>::get_format();
    let depth_format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim,
                                               color_format.0,
                                               depth_format.0);
    let output_color : gfx::handle::RenderTargetView<gfx_device_gl::Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Srgb)> = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);

    let (oscreen_tex, oscreen_srv, oscreen_rtv) = factory.create_render_target::<Srgba8>(
        draw_size.width as u16,
        draw_size.height as u16).unwrap();

    let (oscreen2_tex, oscreen2_srv, oscreen2_rtv) = factory.create_render_target::<Srgba8>(
        draw_size.width as u16,
        draw_size.height as u16).unwrap();

    let (_, _, oscreen_dsv) = factory.create_depth_stencil::<DepthStencil>(
        draw_size.width as u16,
        draw_size.height as u16).unwrap();

    let sinfo = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp);

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&QUAD_VERTS, ());
    let (vertex_buffer2, slice2) = factory.create_vertex_buffer_with_slice(&QUAD_VERTS, ());

	let pso_1 = factory.create_pipeline_simple(
		include_bytes!("../assets/Yasqueen.glslv"),
		include_bytes!("../assets/Yasqueen.glslf"),
		pipe::new()).unwrap();

    let pso_2 = factory.create_pipeline_simple(
		include_bytes!("../assets/Roflcopter.glslv"),
		include_bytes!("../assets/Roflcopter.glslf"),
		pipe::new()).unwrap();

    // Write to second offscreen target using first offscreen as input
	let mut data_1 = pipe::Data {
		vbuf: vertex_buffer,
        tex: (oscreen_srv, factory.create_sampler(sinfo)),
		out: oscreen2_rtv,
	};

    // Write to framebuffer target using second offscreen as input
    let mut data_2 = pipe::Data {
		vbuf: vertex_buffer2,
        tex: (oscreen2_srv, factory.create_sampler(sinfo)),
		out: output_color.clone(),
	};

    let mut encoder = factory.create_command_buffer().into();
    let mut g2d = Gfx2d::new(opengl, &mut factory);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let mut glyph_cache = GlyphCache::new(
        assets.join("FiraSans-Regular.ttf"),
        factory.clone(),
        TextureSettings::new()).unwrap();

    let mut assets = Assets {
        radioactive: Texture::from_path(
            &mut factory,
            assets.join("Radioactive.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap(),
        starting: Texture::from_path(
            &mut factory,
            assets.join("Starting.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap(),
        game_over: Texture::from_path(
            &mut factory,
            assets.join("GameOver.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap(),
        pigeon_points_f0: Vec::new(),
        pigeon_points_f1: Vec::new(),
    };

    let mut game_state = GameState {
        pigeons: Vec::new(),
        coops: Vec::new(),
        bubbles: Vec::new(),
        system_hubs: SystemHubCollection::new(),
        irradiance_field: ScalarField::new(16 * 4, 9 * 4),
        aim_trajectory: Trajectory { points: Vec::new() },
        game_over: false,
        game_over_time: 0.0,
        starting: true,
        selected_coop: None,
        pigeon_timer: 0.0,
    };

    on_load(&mut assets, &mut game_state);

	let mut cursor = Point::new(0f32, 0f32);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            g2d.draw(&mut encoder, &oscreen_rtv, &oscreen_dsv, args.viewport(), |c, g| {
                let mut render_state = RenderState { g, c };
                render_irradiance(&mut factory, &assets, &game_state, &mut render_state, &args);
                render_hubs(&assets, &game_state, &mut render_state, &args);
                render_radiation(&game_state, &mut render_state, &game_state.irradiance_field, game_state.pigeon_timer);
                render_trajectory(&assets, &game_state, &mut render_state, &args);
                render_coop(&assets, &game_state, &mut render_state, &args);
                render_pigeons(&assets, &game_state, &mut render_state, &args);
            });

            // Chromatic abomination
            encoder.draw(&slice, &pso_1, &data_1);

            // Gouging blur
            encoder.draw(&slice2, &pso_2, &data_2);

            if game_state.starting || game_state.game_over {
                encoder.clear(&data_2.out, [0.0, 0.0, 0.0, 1.0]);
            }

            // Render you-eye without post
            g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
                let mut render_state = RenderState { g, c };
                render_ui(&assets, &game_state, &mut render_state, &args);
            });

            // Weep, Mr. GPU
            encoder.flush(&mut device);
        }

        if let Some(u) = e.update_args() {
            on_update(&mut game_state, &u, cursor);
        }

        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }

        // Update coop pigeon emission
        if let Some(Button::Mouse(button)) = e.press_args() {
            if !game_state.starting {
                on_mouse_click(&mut game_state, [cursor.x as f64, cursor.y as f64]);
            }
        }

        if let Some(Button::Mouse(_)) = e.release_args() {
            if game_state.starting {
                game_state.starting = false;
            } else if game_state.game_over{
            	if game_state.pigeon_timer - game_state.game_over_time > 1.0 {
	                game_state.game_over = false;
	                game_state.starting = true;

	                game_state = GameState {
	                    pigeons: Vec::new(),
	                    coops: Vec::new(),
	                    bubbles: Vec::new(),
	                    system_hubs: SystemHubCollection::new(),
	                    irradiance_field: ScalarField::new(16 * 4, 9 * 4),
	                    aim_trajectory: Trajectory { points: Vec::new() },
	                    game_over: false,
	                    game_over_time: 0.0,
	                    starting: true,
	                    selected_coop: None,
	                    pigeon_timer: 0.0,
	                };

	                on_load(&mut assets, &mut game_state);
	            }
            }
            else {
                on_mouse_release(&mut game_state, [cursor.x as f64, cursor.y as f64]);
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::C {
                //play_pigeon_sound();
            }

            if key == Key::G {
                //on_game_over(&mut game_state);
            }

            if key == Key::R {

            }

            if key == Key::S {
                play_camera_shake();
            }
        }

        e.mouse_cursor(|x, y| {
            cursor = Point::new(x as f32, y as f32);
			cursor = cursor
			/ Point::new(window.size().width as f32, window.size().height as f32)
			- Point::new(0.5f32, 0.5f32);
			cursor = cursor * Point::new(16.0 / 9.0 * 2.0, -2.0);

            on_mouse_move(&mut game_state, [cursor.x as f64, cursor.y as f64]);
        });

        e.text(|_text| {
            //println!("Typed '{}'", text)
        });

        e.resize(|_w, _h| {
            //println!("Resized '{}, {}'", w, h)
        });
    }
}