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
use piston::window::{ AdvancedWindow, /*Window, */WindowSettings };
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

use std::io::BufReader;
use models::pigeon::Pigeon;
use models::coop::Coop;
use geometry::traits::Collide;

pub struct PigeonTrajectory {
	points: Vec<Point>
}

pub struct RenderState {
    gl: GlGraphics, // OpenGL drawing backend.
}

pub struct GameState {
    rotation: f64,   // Rotation for the square
    pigeons: Vec<Pigeon>,
    coops: Vec<Coop>,
    irradiance_field: ScalarField,
    aim_trajectory: PigeonTrajectory,
}

pub struct Game<'a> {
    render_state: RenderState,
    game_state: GameState,
    glyph_cache: GlyphCache<'a>
}

impl<'a> Game<'a> {
    fn new(glyphs: GlyphCache<'a>) -> Game<'a> {
		let mut sf = ScalarField::new(16 * 4, 9 * 4);
		sf.splat(20, 15, 9f32);
		sf.splat(40, 30, 7f32);

        Game {
            render_state: RenderState { gl: GlGraphics::new(OpenGL::V3_2) },
            game_state: GameState {
            	rotation: 0.0,
            	pigeons: Vec::new(),
            	coops: Vec::new(),
            	irradiance_field: sf,
            	aim_trajectory: PigeonTrajectory { points: Vec::new() },
            },
            glyph_cache: glyphs
        }
    }

    fn on_load(&mut self, _w: &GameWindow) {
        println!("Adding pigeons!");
        let pos = geometry::Vector {
            position: geometry::Point::new(400.0, 400.0),
            direction: 0.0
        };
        self.game_state.pigeons.push(Pigeon::new(pos));

        let pos_coop = geometry::Point::new(200.0, 200.0);
        self.game_state.coops.push(Coop::new(pos_coop));
    }

    fn simulate_trajectory(&mut self, mouse_x: f64, mouse_y: f64) {
    	let mut pos = Point::new(0.5f32, 0.0f32);
    	//let mut vel = Point::new(0.0f32, 1.0f32);

    	// HACK: todo: un-hardcode the screen resolution
    	let mut vel = Point::new((mouse_x as f32 - 1920f32 * 0.5f32), 1080f32 - mouse_y as f32).normalized();
    	//let mut vel = vel * 0.1f32;

    	let points = &mut self.game_state.aim_trajectory.points;
    	points.clear();

    	let iter_count = 70;
    	let delta_t = 0.03f32;

    	for _ in 0..iter_count {
    		points.push(pos);
    		let grad = self.game_state.irradiance_field.sample_gradient(pos.x, pos.y);
    		vel = vel * 0.98 + grad * 0.23;

    		pos = pos + vel * delta_t;
    	}
    }

    fn update(&mut self, args: &UpdateArgs, mouse_x: f64, mouse_y: f64) {
        // Rotate 2 radians per second.
        self.game_state.rotation += 2.0 * args.dt;
        self.simulate_trajectory(mouse_x, mouse_y);
    }

    fn render_pigeon(render_state: &mut RenderState, game_state: &GameState, args: &RenderArgs, _pigeon: &Pigeon) {
        use graphics::*;
        use geometry::traits::Position;

        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        render_state.gl.draw(args.viewport(), |c, gl| {
            let square = graphics::rectangle::square(0.0, 0.0, 50.0);
            let rotation = game_state.rotation;

            //let (x, y) = ((args.width  / 2) as f64,
             //             (args.height / 2) as f64);

            let transform = c.transform.trans(_pigeon.x() as f64, _pigeon.y() as f64)
                                        .rot_rad(rotation)
                                        .trans(-25.0, -25.0);
            graphics::rectangle(BLUE, square, transform, gl);
        });
    }

    fn render_coop(render_state: &mut RenderState, args: &RenderArgs, _coop: &Coop) {
        use graphics::*;
        use geometry::traits::Position;

        const ORANGE:  [f32; 4] = [1.0, 0.5647, 0.0039, 1.0];
        use graphics::math::Vec2d;

        render_state.gl.draw(args.viewport(), |c, gl| {

            let mut transform = c.transform.trans(_coop.x() as f64, _coop.y() as f64)
                                        .scale(_coop.radius() as f64, _coop.radius() as f64);

            let mut pentagon: [Vec2d;5] = [[1.0,0.0], [0.309, -0.951], [-0.809, -0.588], [-0.809, 0.588], [0.309, 0.951]];
            if let Some(dir) = _coop.direction {
                pentagon[0][0] *= 2.0;
                transform = transform.rot_rad(dir as f64);
            }

            graphics::polygon(ORANGE, &pentagon, transform, gl);
        });
    }

    fn render_trajectory(gl: &mut opengl_graphics::GlGraphics, trajectory: &PigeonTrajectory) {
    	let scale_0_to_1 = graphics::math::identity().trans(-1.0, -1.0).scale(2.0, 2.0);
    	if trajectory.points.len() < 2 { 
    		return;
    	}

    	use graphics::*;

    	for i in 1..trajectory.points.len() {
	    	Line::new([1.0f32, 1.0f32, 1.0f32, 1.0f32], 0.001).draw([
	    		trajectory.points[i-1].x as f64,
	    		trajectory.points[i-1].y as f64,
	    		trajectory.points[i].x as f64,
	    		trajectory.points[i].y as f64,
	    	], &Default::default(), scale_0_to_1, gl);
	    }
    }

    fn render(render_state: &mut RenderState, game_state: &GameState, glyph_cache: &mut GlyphCache, args: &RenderArgs, mouse_x: f64, mouse_y: f64) {

        use graphics::*;
        let mouse_square = rectangle::square(0.0, 0.0, 50.0);

        render_state.gl.draw(args.viewport(), |c, gl| {
        	let sf = &game_state.irradiance_field;
	        let sf_texture = Texture::from_image(
	            &sf.to_image_buffer(),
	            &TextureSettings::new()
	        );

	        let scale_0_to_1 = graphics::math::identity().trans(-1.0, -1.0).scale(2.0, 2.0);

            Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(
			    &sf_texture,
			    &Default::default(),
			    scale_0_to_1.scale(1.0 / sf.width as f64, 1.0 / sf.height as f64),
			    gl
			);

            // Test line rendering
            // Line::new([1.0, 1.0, 1.0, 1.0], 0.001).draw([0f64, 0f64, 1f64, 1f64], &Default::default(), scale_0_to_1, gl);

            Game::render_trajectory(gl, &game_state.aim_trajectory);

            let rotation = game_state.rotation;
            let mouse_transform = c.transform.trans(mouse_x, mouse_y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            rectangle(RED, mouse_square, mouse_transform, gl);

            text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw("PIGEON SIMULATOR 2018",
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
    }

    fn on_mouse_move(&mut self, mouse: [f64;2]) {
        // Update coop pigeon shooting directions
        for mut coop in self.game_state.coops.iter_mut() {
            Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));
        }
    }

    fn on_mouse_click(&mut self, mouse: [f64;2]) {
        // Select coop if clicking inside
        for mut coop in self.game_state.coops.iter_mut() {
            Coop::update_mouse_click(coop, Point::new(mouse[0] as f32, mouse[1] as f32));
        }
    }

    fn on_mouse_release(&mut self) {
        // Shoot pigeon if mouse button is released
        for mut coop in self.game_state.coops.iter_mut() {
            if let Some(pigeon) = Coop::update_mouse_release(coop) {
                self.game_state.pigeons.push(pigeon);
            }
        }
    }
}

fn main() {

    println!("GGJ-Base");
    
    //let opengl = OpenGL::V3_2;

    let mut window: GameWindow = WindowSettings::new(
            "ggj-base",
            [1920, 1080]
        )
        .opengl(OpenGL::V3_2)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor = [0.0, 0.0];

    //let assets = find_folder::Search::ParentsThenKids(3, 3)
    //    .for_folder("assets").unwrap();

    let glyph_cache = GlyphCache::new("assets/FiraSans-Regular.ttf", (), TextureSettings::new()).unwrap();
    let mut game = Game::new(glyph_cache);
    game.on_load(&window);

    // http://blog.piston.rs/2014/09/13/rust-event/

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            Game::render(&mut game.render_state, &game.game_state, &mut game.glyph_cache, &r, cursor[0], cursor[1]);
        }

        if let Some(u) = e.update_args() {
            game.update(&u, cursor[0], cursor[1]);
        }

        // Update coop pigeon emission
        if let Some(Button::Mouse(button)) = e.press_args(){
            game.on_mouse_click(cursor);

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
        }

        e.mouse_cursor(|x, y| {
            cursor = [x, y];
            game.on_mouse_move(cursor);
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
    let x: u32 = rng.gen_range(1,4);
    let s: String = x.to_string();
    let ss: &str = &s;

    let mut sound_file = String::from("assets/coo");
    sound_file.push_str(ss);
    sound_file.push_str(".wav");
    play_sound(&sound_file);
}