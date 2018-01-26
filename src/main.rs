extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate freetype as ft;
extern crate find_folder;
extern crate nalgebra;
extern crate ncollide;
extern crate ai_behavior;
extern crate sprite;

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
use opengl_graphics::{ GlGraphics, Texture, TextureSettings, OpenGL };
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

pub struct Game<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,   // Rotation for the square
    face: ft::Face<'a>,
}

impl<'a> Game<'a> {
    fn new(face: ft::Face<'a>) -> Game<'a> {
        Game { gl: GlGraphics::new(OpenGL::V4_1), rotation: 0.0, face }
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

        let mut face = &mut self.face;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);

            render_text(&mut face, gl, transform, "Hello Piston!");
        });
    }


}

fn render_text(face: &mut ft::Face, gl: &mut GlGraphics, t: Matrix2d, text: &str) {
    let mut x = 10;
    let mut y = 0;
    for ch in text.chars() {
        use graphics::*;

        face.load_char(ch as usize, ft::face::LoadFlag::RENDER).unwrap();
        let g = face.glyph();

        let bitmap = g.bitmap();
        let texture = Texture::from_memory_alpha(
            bitmap.buffer(),
            bitmap.width() as u32,
            bitmap.rows() as u32,
            &TextureSettings::new()
        ).unwrap();
        Image::new_color(color::BLACK).draw(
            &texture,
            &Default::default(),
            t.trans((x + g.bitmap_left()) as f64, (y - g.bitmap_top()) as f64),
            gl
        );

        x += (g.advance().x >> 6) as i32;
        y += (g.advance().y >> 6) as i32;
    }
}

fn main() {

    println!("GGJ-Base");

    //let opengl = OpenGL::V4_1;

    let mut window: GameWindow = WindowSettings::new(
            "ggj-base",
            [1920, 1080]
        )
        .opengl(OpenGL::V4_1)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut capture_cursor = false;
    let mut cursor = [0.0, 0.0];

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let freetype = ft::Library::init().unwrap();
    let font = assets.join("FiraSans-Regular.ttf");
    let face = freetype.new_face(&font, 0).unwrap();
    face.set_pixel_sizes(0, 48).unwrap();

    let mut game = Game::new(face);
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