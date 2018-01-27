#![allow(dead_code)]

/*
extern crate piston;
extern crate graphics;
//extern crate opengl_graphics;
extern crate find_folder;
extern crate nalgebra;
extern crate ncollide;
extern crate ai_behavior;
extern crate sprite;
extern crate rand;
extern crate image;
extern crate rodio;
extern crate gfx_graphics;
extern crate gfx;
extern crate gfx_device_gl;*/

//#[cfg(feature="piston")] #[macro_use] extern crate conrod;
//#[cfg(feature="piston")] mod support;
//pub mod object;
//use object::Object;

//mod scalar_field;
//use scalar_field::*;

//#[macro_use]
//pub mod geometry;
//use geometry::point::{Point};

//#[macro_use]
//pub mod models;

//#[allow(unused_imports)]
//use piston::window::{ OpenGLWindow, AdvancedWindow, Window, WindowSettings };
//use piston::event_loop::*;
//use piston::input::*;

//#[allow(unused_imports)]
//use opengl_graphics::{ GlGraphics, Texture, TextureSettings, OpenGL, GlyphCache };

//#[allow(unused_imports)]
//use graphics::math::Matrix2d;


//use models::pigeon::*;
//use models::coop::Coop;
//use geometry::traits::Collide;

extern crate glutin_window;
extern crate piston;
extern crate graphics;
extern crate gfx_graphics;
extern crate find_folder;
extern crate gfx;
extern crate gfx_device_gl;
extern crate rodio;
extern crate rand;

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

use GfxGraphics<'_, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> as GraphicsObj;


pub struct RenderState<'a> {
    g: &'a mut GfxGraphics<'a, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
}

/*
pub struct GameState {
    rotation: f64,   // Rotation for the square
    pigeons: Vec<Pigeon>,
    coops: Vec<Coop>,
    irradiance_field: ScalarField,
    aim_trajectory: Trajectory,
    selected_coop: Option<usize>,
    game_over: bool,
    pigeon_f0: bool,
    pigeon_timer: f64,
}

pub struct Assets {
    game_over: Texture,
    pigeon_points_f0: Vec<(geometry::Point, geometry::Point)>,
    pigeon_points_f1: Vec<(geometry::Point, geometry::Point)>,
}

pub struct Game<'a> {
    render_state: RenderState,
    game_state: GameState,
    glyph_cache: GlyphCache<'a>,
    assets: Assets,
}

fn pos_to_irradiance_coord(p: Point) -> Point {
	let aspect = 16.0 / 9.0;
	(p + Point::new(aspect, 1.0)) / Point::new(aspect * 2.0, 2.0)
}

impl<'a> Game<'a> {
    fn new(glyphs: GlyphCache<'a>) -> Game<'a> {
		let mut sf = ScalarField::new(16 * 4, 9 * 4);

        Game {
            render_state: RenderState { gl: GlGraphics::new(OpenGL::V3_2) },
            game_state: GameState {
            	rotation: 0.0,
            	pigeons: Vec::new(),
            	coops: Vec::new(),
            	irradiance_field: sf,
            	aim_trajectory: Trajectory { points: Vec::new() },
            	game_over: false,
                pigeon_f0: true,
            	selected_coop: None,
                pigeon_timer: 0.0,
            },
            glyph_cache: glyphs,
            assets: Assets {
                game_over: Texture::from_path(
                    &Path::new("./assets/GameOver.png"),
                    &TextureSettings::new()
                ).unwrap(),
                pigeon_points_f0: Vec::new(),
                pigeon_points_f1: Vec::new(),
            }
        }
    }

    fn on_load(&mut self, _w: &GameWindow) {
        let pos_coop = geometry::Point::new(0.0, -0.7);
        self.game_state.coops.push(Coop::new(pos_coop));

        // Pigeon animation frame #0
        self.assets.pigeon_points_f0.push((Point::new(400.0, 442.043),   Point::new(100.0, 442.043)));
        self.assets.pigeon_points_f0.push((Point::new(100.0, 442.043),   Point::new(250.443, 57.113)));
        self.assets.pigeon_points_f0.push((Point::new(250.443, 57.113),  Point::new(400.0, 442.043)));
        self.assets.pigeon_points_f0.push((Point::new(309.156, 205.907), Point::new(445.678, 147.404)));
        self.assets.pigeon_points_f0.push((Point::new(445.678, 147.404), Point::new(375.655, 376.547)));
        self.assets.pigeon_points_f0.push((Point::new(375.655, 376.547), Point::new(309.156, 205.907)));
        self.assets.pigeon_points_f0.push((Point::new(191.678, 205.907), Point::new(55.156, 147.404)));
        self.assets.pigeon_points_f0.push((Point::new(55.156, 147.404),  Point::new(125.179, 376.547)));
        self.assets.pigeon_points_f0.push((Point::new(125.179, 376.547), Point::new(191.678, 205.907)));

        // Pigeon animation frame #1
        self.assets.pigeon_points_f1.push((Point::new(400.0, 442.043), Point::new(100.0, 442.043)));
        self.assets.pigeon_points_f1.push((Point::new(100.0, 442.043), Point::new(250.443, 57.113)));
        self.assets.pigeon_points_f1.push((Point::new(250.443, 57.113), Point::new(400.0, 442.043)));
        self.assets.pigeon_points_f1.push((Point::new(308.156, 205.907), Point::new(449.411, 311.876)));
        self.assets.pigeon_points_f1.push((Point::new(449.411, 311.876), Point::new(374.655, 376.547)));
        self.assets.pigeon_points_f1.push((Point::new(374.655, 376.547), Point::new(308.156, 205.907)));
        self.assets.pigeon_points_f1.push((Point::new(192.411, 205.907), Point::new(51.156, 311.876)));
        self.assets.pigeon_points_f1.push((Point::new(51.156, 311.876), Point::new(125.912, 376.547)));
        self.assets.pigeon_points_f1.push((Point::new(125.912, 376.547), Point::new(192.411, 205.907)));
    }

    fn simulate_trajectory(&self, origin: Point, cursor: Point) -> Trajectory {
    	let mut pos = origin;
    	let mut vel = cursor - pos;
    	//let mut vel = vel * 0.1f32;

    	let mut points = Vec::new();

    	let iter_count = 200;
    	let delta_t = 0.07f32;

    	for _ in 0..iter_count {
    		points.push(pos);
    		let grad = self.game_state.irradiance_field.sample_gradient(pos_to_irradiance_coord(pos));
    		vel = vel * 0.98 + grad * 0.23;

    		pos = pos + vel * delta_t;

    		// TODO: x bounds
    		if pos.y > 1f32 || pos.y < -1f32 {
    			break;
    		}
    	}

    	Trajectory { points }
    }

    fn update(&mut self, args: &UpdateArgs, cursor: Point) {
        // Rotate 2 radians per second.
        self.game_state.rotation += 2.0 * args.dt;

        // Radioactive decay
        self.game_state.irradiance_field.decay(0.998f32);

        // Fixed radiation source for the reactor or whatever
		self.game_state.irradiance_field.splat(pos_to_irradiance_coord(Point::new(0f32, 0.5f32)), 7f32, RadiationBlendMode::Max);

        let mut pigeon_to_nuke = None;
        for i in 0..self.game_state.pigeons.len() {
        	let mut pigeon = &mut self.game_state.pigeons[i];
            if let PigeonStatus::ReachedDestination = pigeon.update((1.0 * args.dt) as f32) {
            	pigeon_to_nuke = Some(i);
            }
        }

        if let Some(i) = pigeon_to_nuke {
        	let pos = self.game_state.pigeons[i].vector.position;
        	self.game_state.pigeons.swap_remove(i);
        	self.game_state.irradiance_field.splat(pos_to_irradiance_coord(pos), 4f32, RadiationBlendMode::Add);
        }

        if let Some(coop_idx) = self.game_state.selected_coop {
        	self.game_state.aim_trajectory =
        		self.simulate_trajectory(self.game_state.coops[coop_idx].position, cursor);
        }

        self.game_state.pigeon_timer += args.dt;
    }

    fn render_pigeon(assets: &Assets, render_state: &mut RenderState, game_state: &GameState, args: &RenderArgs, pigeon: &Pigeon) {
        use graphics::*;
        use geometry::traits::Position;

        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        render_state.gl.draw(args.viewport(), |_c, gl| {
            let square = graphics::rectangle::square(0.0, 0.0, 0.1);
            let rotation = pigeon.vector.direction as f64;

            //let (x, y) = ((args.width  / 2) as f64,
             //             (args.height / 2) as f64);



            let transform = Game::std_transform()
				.trans(pigeon.x() as f64, pigeon.y() as f64)
				.rot_rad(rotation);

            let remapped = (game_state.pigeon_timer * 40.0).sin() * 0.5 + 0.5; // -1...1 -> 0...1

            let scale = 0.00023;
            for i in 0..assets.pigeon_points_f0.len() {
                let (s_o, e_o) = assets.pigeon_points_f0[i];
                let (s_p, e_p) = assets.pigeon_points_f1[i];
                let animated_s = s_o.lerp(&s_p, remapped as f32);
                let animated_e = e_o.lerp(&e_p, remapped as f32);
                graphics::Line::new([1.0f32, 1.0f32, 1.0f32, 1.0f32], 0.002).draw([
                    (animated_s.x - 256.0) as f64 * scale,
                    (animated_s.y - 256.0) as f64 * -scale,
                    (animated_e.x - 256.0) as f64 * scale,
                    (animated_e.y - 256.0) as f64 * -scale,
                ], &Default::default(), transform, gl);
            }
        });
    }

    fn render_coop(render_state: &mut RenderState, args: &RenderArgs, _coop: &Coop) {
        use graphics::*;
        use geometry::traits::Position;

        const color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        use graphics::math::Vec2d;

        render_state.gl.draw(args.viewport(), |_c, gl| {

            let mut transform = Game::std_transform()
				.trans(_coop.x() as f64, _coop.y() as f64)
				.scale(_coop.radius() as f64, _coop.radius() as f64);

            let mut pentagon: [Vec2d;5] = [[1.0,0.0], [0.309, -0.951], [-0.809, -0.588], [-0.809, 0.588], [0.309, 0.951]];
            if let Some(dir) = _coop.direction {
                pentagon[0][0] *= 2.0;
                transform = transform.rot_rad(dir as f64);
            }

            graphics::polygon(color, &pentagon, transform, gl);
        });
    }

    fn std_transform() -> Matrix2d {
    	use graphics::*;
    	let aspect = 16.0 / 9.0;
    	graphics::math::identity().scale(1.0 / aspect, 1.0)
    }

    fn render_trajectory(gl: &mut opengl_graphics::GlGraphics, trajectory: &Trajectory) {
    	if trajectory.points.len() < 2 {
    		return;
    	}

    	use graphics::*;

    	for i in 1..trajectory.points.len() {
	    	Line::new([1.0f32, 0.1f32, 0.02f32, 1.0f32], 0.005).draw([
	    		trajectory.points[i-1].x as f64,
	    		trajectory.points[i-1].y as f64,
	    		trajectory.points[i].x as f64,
	    		trajectory.points[i].y as f64,
	    	], &Default::default(), Game::std_transform(), gl);
	    }
    }

    fn render(assets: &Assets, render_state: &mut RenderState, game_state: &GameState, glyph_cache: &mut GlyphCache, args: &RenderArgs, _cursor: Point) {
        use graphics::*;
        let _mouse_square = rectangle::square(0.0, 0.0, 0.1);
        let scale_0_to_1 = graphics::math::identity().trans(-1.0, -1.0).scale(2.0, 2.0);

        render_state.gl.draw(args.viewport(), |c, gl| {
        	let sf = &game_state.irradiance_field;
	        let sf_texture = Texture::from_image(
	            &sf.to_image_buffer(),
	            &TextureSettings::new()
	        );

            Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(
			    &sf_texture,
			    &Default::default(),
			    scale_0_to_1.scale(1.0 / sf.width as f64, 1.0 / sf.height as f64),
			    gl
			);

            // Test line rendering
            // Line::new([1.0, 1.0, 1.0, 1.0], 0.001).draw([0f64, 0f64, 1f64, 1f64], &Default::default(), scale_0_to_1, gl);

            if let Some(_) = game_state.selected_coop {
            	Game::render_trajectory(gl, &game_state.aim_trajectory);
            }

            //let rotation = game_state.rotation;
            //let mouse_transform = Game::std_transform()
            //	.trans(cursor.x as f64, cursor.y as f64)
			//	.rot_rad(rotation)//.trans(-25.0, -25.0)
			//	;

            // Draw a box rotating around the middle of the screen.
            //const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            //rectangle(RED, mouse_square, mouse_transform, gl);

            text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw("IRRADIANT DESCENT",
                                                                     glyph_cache,
                                                                     &DrawState::default(),
                                                                     c.transform
                                                                         .trans(10.0, 100.0),
                                                                     gl).unwrap();
        });

        let pigeons = &game_state.pigeons;
        for pigeon in pigeons.iter() {
            Game::render_pigeon(assets, render_state, game_state, args, pigeon);
        }

        let coops = &game_state.coops;
        for coop in coops.iter() {
            Game::render_coop(render_state, args, coop);
        }

        // Full Screen UI
        if game_state.game_over {
            render_state.gl.draw(args.viewport(), |_c, gl| {
                let gui_transform = scale_0_to_1
                	.flip_v()
                	.trans(0.0, -1.0)
                	.scale(1.0 / assets.game_over.get_width() as f64, 1.0 / assets.game_over.get_height() as f64);
                image(&assets.game_over, gui_transform, gl);
            });
        }
    }

    fn on_mouse_move(&mut self, mouse: [f64;2]) {
        // Update coop pigeon shooting directions
        for mut coop in self.game_state.coops.iter_mut() {
            Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));
        }
    }

    fn on_mouse_click(&mut self, mouse: [f64;2]) {
        // Select coop if clicking inside
        for coop_idx in 0..self.game_state.coops.len() {
        	let mut coop = &mut self.game_state.coops[coop_idx];
            if Coop::update_mouse_click(coop, Point::new(mouse[0] as f32, mouse[1] as f32)) {
            	self.game_state.selected_coop = Some(coop_idx);
            }
        }
    }

    fn on_mouse_release(&mut self) {
        // Shoot pigeon if mouse button is released
        for mut coop in self.game_state.coops.iter_mut() {
            if let Some(mut pigeon) = Coop::update_mouse_release(coop) {
                pigeon.trajectory = Some(self.game_state.aim_trajectory.clone());
                self.game_state.pigeons.push(pigeon);
            }
        }

        self.game_state.selected_coop = None;
    }

    fn on_game_over(&mut self) {
        // Test with toggle
        self.game_state.game_over = !self.game_state.game_over;
    }
}*/

fn main() {

    let opengl = OpenGL::V3_2;
    let samples = 4;
    let mut window: GlutinWindow = WindowSettings::new(
            "piston: draw_state",
            [600, 600]
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
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let blends = [Blend::Alpha, Blend::Add, Blend::Invert, Blend::Multiply];
    let mut blend = 0;
    let mut clip_inside = true;
    let rust_logo = Texture::from_path(&mut factory,
                                       assets.join("rust.png"),
                                       Flip::None,
                                       &TextureSettings::new()).unwrap();

    let mut encoder = factory.create_command_buffer().into();
    let mut g2d = Gfx2d::new(opengl, &mut factory);
    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
                let blah = RenderState<'a> { g };

                clear([0.8, 0.8, 0.8, 1.0], g);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

                let draw_state = c.draw_state.blend(blends[blend]);
                Rectangle::new([0.5, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0], &draw_state, c.transform, g);

                let transform = c.transform.trans(100.0, 100.0);
                // Compute clip rectangle from upper left corner.
                let (clip_x, clip_y, clip_w, clip_h) = (100, 100, 100, 100);
                let (clip_x, clip_y, clip_w, clip_h) =
                    (clip_x, c.viewport.unwrap().draw_size[1] - clip_y - clip_h, clip_w, clip_h);
                let clipped = c.draw_state.scissor([clip_x, clip_y, clip_w, clip_h]);
                Image::new().draw(&rust_logo, &clipped, transform, g);

                let transform = c.transform.trans(200.0, 200.0);
                Ellipse::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 50.0, 50.0], &DrawState::new_clip(), transform, g);
                Image::new().draw(&rust_logo,
                    &if clip_inside { DrawState::new_inside() }
                    else { DrawState::new_outside() },
                    transform, g);
            });
            encoder.flush(&mut device);
        }

        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = (blend + 1) % blends.len();
            println!("Changed blending to {:?}", blends[blend]);
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            clip_inside = !clip_inside;
            if clip_inside {
                println!("Changed to clip inside");
            } else {
                println!("Changed to clip outside");
            }
        }
    }

/*
    let mut window: GameWindow = WindowSettings::new(
            "Irradiant Descent",
            [1920, 1080]
        )
        .opengl(OpenGL::V3_2)
        .exit_on_esc(true)
        .build()
        .unwrap();

	let mut cursor = Point::new(0f32, 0f32);
    let glyph_cache = GlyphCache::new("assets/FiraSans-Regular.ttf", (), TextureSettings::new()).unwrap();
    let mut game = Game::new(glyph_cache);
    game.on_load(&window);

    // http://blog.piston.rs/2014/09/13/rust-event/

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            Game::render(&game.assets, &mut game.render_state, &game.game_state, &mut game.glyph_cache, &r, cursor);
        }

        if let Some(u) = e.update_args() {
            game.update(&u, cursor);
        }

        // Update coop pigeon emission
        if let Some(Button::Mouse(button)) = e.press_args(){
            game.on_mouse_click([cursor.x as f64, cursor.y as f64]);

            if button == MouseButton::Left {
                play_sound("assets/dummy.wav");
            }
            else if button == MouseButton::Right {
                play_sound("assets/footstep.wav");
            }
        }

        if let Some(Button::Mouse(_)) = e.release_args() {
            game.on_mouse_release();
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::C {
                pigeon_sound();
            }

            if key == Key::G {
                game.on_game_over();
            }
        }

        e.mouse_cursor(|x, y| {
            cursor = Point::new(x as f32, y as f32);
			cursor = cursor
			/ Point::new(window.size().width as f32, window.size().height as f32)
			- Point::new(0.5f32, 0.5f32);
			cursor = cursor * Point::new(16.0 / 9.0 * 2.0, -2.0);

            game.on_mouse_move([cursor.x as f64, cursor.y as f64]);
        });

        e.text(|text| println!("Typed '{}'", text));
        e.resize(|w, h| println!("Resized '{}, {}'", w, h));
    }
    */
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

fn pigeon_sound()
{
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x: u32 = rng.gen_range(1,13);
    let s: String = x.to_string();
    let ss: &str = &s;

    let mut sound_file = String::from("assets/coo");
    sound_file.push_str(ss);
    sound_file.push_str(".wav");
    play_sound(&sound_file);
}