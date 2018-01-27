#![allow(dead_code)]

extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate find_folder;
extern crate nalgebra;
extern crate ncollide;
extern crate ai_behavior;
extern crate sprite;
extern crate rand;
extern crate image;
extern crate rodio;

#[cfg(feature="piston")] #[macro_use] extern crate conrod;
#[cfg(feature="piston")] mod support;

#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;
#[cfg(feature = "include_glfw")]
extern crate glfw_window;
#[cfg(feature = "include_glutin")]
extern crate glutin_window;

pub mod object;
//use object::Object;

mod scalar_field;
use scalar_field::*;

#[macro_use]
pub mod geometry;
use geometry::point::{Point};

#[macro_use]
pub mod models;

#[allow(unused_imports)]
use piston::window::{ AdvancedWindow, Window, WindowSettings };
use piston::event_loop::*;
use piston::input::*;

#[allow(unused_imports)]
use opengl_graphics::{ GlGraphics, Texture, TextureSettings, OpenGL, GlyphCache };

#[allow(unused_imports)]
use graphics::math::Matrix2d;
//use sprite::*;
/*use ai_behavior::{
    Action,
    Sequence,
    Wait,
    WaitForever,
    While,
};*/

#[cfg(feature = "include_sdl2")]
use sdl2_window::Sdl2Window as GameWindow;
#[cfg(feature = "include_glfw")]
use glfw_window::GlfwWindow as GameWindow;
#[cfg(feature = "include_glutin")]
use glutin_window::GlutinWindow as GameWindow;

use std::path::Path;
use std::io::BufReader;
use models::pigeon::*;
use models::coop::Coop;
use geometry::traits::Collide;

pub struct RenderState {
    gl: GlGraphics, // OpenGL drawing backend.
}

pub struct GameState {
    rotation: f64,   // Rotation for the square
    pigeons: Vec<Pigeon>,
    coops: Vec<Coop>,
    irradiance_field: ScalarField,
    aim_trajectory: Trajectory,
    selected_coop: Option<usize>,
    game_over: bool,
}

pub struct Assets {
    game_over: Texture
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
		sf.splat(pos_to_irradiance_coord(Point::new(0f32, 0f32)), 9f32);

        Game {
            render_state: RenderState { gl: GlGraphics::new(OpenGL::V3_2) },
            game_state: GameState {
            	rotation: 0.0,
            	pigeons: Vec::new(),
            	coops: Vec::new(),
            	irradiance_field: sf,
            	aim_trajectory: Trajectory { points: Vec::new() },
            	game_over: false,
            	selected_coop: None,
            },
            glyph_cache: glyphs,
            assets: Assets {
                game_over: Texture::from_path(
                    &Path::new("./assets/GameOver.png"),
                    &TextureSettings::new()
                ).unwrap()
            }
        }
    }

    fn on_load(&mut self, _w: &GameWindow) {
        let pos_coop = geometry::Point::new(0.0, -0.7);
        self.game_state.coops.push(Coop::new(pos_coop));
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
        self.game_state.irradiance_field.decay(0.998f32);

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
        	self.game_state.irradiance_field.splat(pos_to_irradiance_coord(pos), 4f32);
        }

        if let Some(coop_idx) = self.game_state.selected_coop {
        	self.game_state.aim_trajectory =
        		self.simulate_trajectory(self.game_state.coops[coop_idx].position, cursor);
        }
    }

    fn render_pigeon(render_state: &mut RenderState, game_state: &GameState, args: &RenderArgs, _pigeon: &Pigeon) {
        use graphics::*;
        use geometry::traits::Position;

        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        render_state.gl.draw(args.viewport(), |_c, gl| {
            let square = graphics::rectangle::square(0.0, 0.0, 0.1);
            let rotation = game_state.rotation;

            //let (x, y) = ((args.width  / 2) as f64,
             //             (args.height / 2) as f64);

            let transform = Game::std_transform()
				.trans(_pigeon.x() as f64, _pigeon.y() as f64)
				.rot_rad(rotation)
				.trans(-0.05, -0.05);
            graphics::rectangle(BLUE, square, transform, gl);
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

    fn render(_assets: &Assets, render_state: &mut RenderState, game_state: &GameState, glyph_cache: &mut GlyphCache, args: &RenderArgs, _cursor: Point) {
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

            /*let rotation = game_state.rotation;
            let mouse_transform = Game::std_transform()
            	.trans(cursor.x as f64, cursor.y as f64)
				.rot_rad(rotation)//.trans(-25.0, -25.0)
				;

            // Draw a box rotating around the middle of the screen.
            const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            rectangle(RED, mouse_square, mouse_transform, gl);*/

            text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw("IRRADIANT DESCENT",
                                                                     glyph_cache,
                                                                     &DrawState::default(),
                                                                     c.transform
                                                                         .trans(10.0, 100.0),
                                                                     gl).unwrap();
        });

        let pigeons = &game_state.pigeons;
        for pigeon in pigeons.iter() {
            Game::render_pigeon(render_state, game_state, args, pigeon);
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
                	.scale(1.0 / _assets.game_over.get_width() as f64, 1.0 / _assets.game_over.get_height() as f64);
                image(&_assets.game_over, gui_transform, gl);
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
}

fn main() {

    println!("GGJ-Base");

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