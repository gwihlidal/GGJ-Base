extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate find_folder;
extern crate nalgebra;
extern crate ncollide;
extern crate ai_behavior;
extern crate sprite;
extern crate rand;

#[cfg(feature="piston")] #[macro_use] extern crate conrod;
#[cfg(feature="piston")] mod support;

#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;
#[cfg(feature = "include_glfw")]
extern crate glfw_window;
#[cfg(feature = "include_glutin")]
extern crate glutin_window;

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

mod object;
//use object::Object;

#[macro_use]
mod geometry;

#[macro_use]
mod models;

pub struct Game<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,   // Rotation for the square
}

impl Game {
    fn new() -> Game {
        Game { gl: GlGraphics::new(OpenGL::V3_2), rotation: 0.0 }
    }

    fn on_load(&mut self, _w: &GameWindow) {

    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
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
            game.render(&r);
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