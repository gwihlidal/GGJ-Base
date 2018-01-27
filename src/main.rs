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

#[macro_use]
pub mod models;

use piston::window::{ AdvancedWindow, /*Window, */WindowSettings };
use piston::event_loop::*;
use piston::input::*;

#[allow(unused_imports)]
use opengl_graphics::{ GlGraphics, Texture, TextureSettings, OpenGL };

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

use models::pigeon::Pigeon;

pub struct RenderState {
    gl: GlGraphics // OpenGL drawing backend.
}

pub struct GameState {
    rotation: f64,   // Rotation for the square
    pigeons: Vec<Pigeon>,
}

pub struct Game {
    render_state: RenderState,
    game_state: GameState
}

impl Game {
    fn new() -> Game {
        Game {
            render_state: RenderState { gl: GlGraphics::new(OpenGL::V3_2) },
            game_state: GameState { rotation: 0.0, pigeons: Vec::new() }
        }
    }

    fn on_load(&mut self, _w: &GameWindow) {
        println!("Adding pigeons!");
        let pos = geometry::Vector {
            position: geometry::Point::new(400.0, 400.0),
            direction: 0.0
        };
        self.game_state.pigeons.push(Pigeon::new(pos));
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.game_state.rotation += 2.0 * args.dt;
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

    fn render(render_state: &mut RenderState, game_state: &GameState, args: &RenderArgs, mouse_x: f64, mouse_y: f64) {

        use graphics::*;
        let mouse_square = rectangle::square(0.0, 0.0, 50.0);

        let mut sf = ScalarField::new(16 * 4, 9 * 4);
        sf.splat(10, 10, 7f32);
        sf.splat(40, 30, 7f32);

        let texture = Texture::from_image(
            &sf.to_image_buffer(),
            &TextureSettings::new()
        );

        render_state.gl.draw(args.viewport(), |c, gl| {
            Image::new_color([1.0, 1.0, 1.0, 1.0]).draw(
			    &texture,
			    &Default::default(),
			    graphics::math::identity().trans(-1.0, -1.0).scale(2.0 / sf.width as f64, 2.0 / sf.height as f64),
			    gl
			);

            let rotation = game_state.rotation;
            let mouse_transform = c.transform.trans(mouse_x, mouse_y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            rectangle(RED, mouse_square, mouse_transform, gl);
        });

        let pigeons = &game_state.pigeons;
        for pigeon in pigeons.iter() {
            Game::render_pigeon(render_state, game_state, args, pigeon);
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

    let mut capture_cursor = false;
    let mut cursor = [0.0, 0.0];

    //let assets = find_folder::Search::ParentsThenKids(3, 3)
    //    .for_folder("assets").unwrap();

    let mut game = Game::new();
    game.on_load(&window);

    // http://blog.piston.rs/2014/09/13/rust-event/

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            Game::render(&mut game.render_state, &game.game_state, &r, cursor[0], cursor[1]);
        }

        if let Some(u) = e.update_args() {
            game.update(&u);
        }

        if let Some(cursor) = e.cursor_args() {
            if cursor { println!("Mouse entered"); }
            else { println!("Mouse left"); }
        }

        if let Some(_args) = e.idle_args() {
            // println!("Idle {}", _args.dt);
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::C {
                println!("Turned capture cursor on");
                capture_cursor = !capture_cursor;
                window.set_capture_cursor(capture_cursor);
            }

            println!("Pressed keyboard key '{:?}'", key);
        }

        if let Some(args) = e.button_args() {
            println!("Scancode {:?}", args.scancode);
        }

        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(key) => println!("Released keyboard key '{:?}'", key),
                Button::Mouse(button) => println!("Released mouse button '{:?}'", button),
                Button::Controller(button) => println!("Released controller button '{:?}'", button),
            }
        }

        e.mouse_cursor(|x, y| {
            cursor = [x, y];
            println!("Mouse moved '{} {}'", x, y);
        });
        e.mouse_scroll(|dx, dy| println!("Scrolled mouse '{}, {}'", dx, dy));
        e.mouse_relative(|dx, dy| println!("Relative mouse moved '{} {}'", dx, dy));
        e.text(|text| println!("Typed '{}'", text));
        e.resize(|w, h| println!("Resized '{}, {}'", w, h));
    }
}