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
use models::systemhub::{SystemHubCollection, PigeonAcceptanceLevel};
use geometry::traits::Collide;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<Srgba8> = "Target0",
    }
}

const TRI_VERTS: [Vertex; 3] = [
	Vertex { pos: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
	Vertex { pos: [0.5,  -0.5], color: [0.0, 1.0, 0.0] },
	Vertex { pos: [0.0,   0.5], color: [0.0, 0.0, 1.0] },
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
    pigeon_timer: f64,
}

pub struct Assets {
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

fn std_transform() -> Matrix2d {
    use graphics::*;
    let aspect = 16.0 / 9.0;
    graphics::math::identity().scale(1.0 / aspect, 1.0)
}

fn on_mouse_move(game_state: &mut GameState, mouse: [f64;2]) {
    // Update coop pigeon shooting directions
    //for mut coop in game_state.coops.iter_mut() {
    //    Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));
    //}

    let mut coop = &mut game_state.coops[0];
    Coop::update_mouse_move(coop, Point::new(mouse[0] as f32, mouse[1] as f32));

    for bubble in game_state.bubbles.iter_mut() {
        SpeechBubble::update_mouse_move(bubble, 1.0, Point::new(mouse[0] as f32, mouse[1] as f32));
    }
}

fn on_mouse_click(game_state: &mut GameState, mouse: [f64;2]) {
    // Select coop if clicking inside
//        for coop_idx in 0..game_state.coops.len() {
//        	let mut coop = &mut game_state.coops[coop_idx];
//            if Coop::update_mouse_click(coop, Point::new(mouse[0] as f32, mouse[1] as f32)) {
//            	game_state.selected_coop = Some(coop_idx);
//            }
//        }

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
    // Shoot pigeon if mouse button is released
    /*for mut coop in game_state.coops.iter_mut() {
        if let Some(mut pigeon) = Coop::update_mouse_release(coop) {
            pigeon.trajectory = Some(game_state.aim_trajectory.clone());
            game_state.pigeons.push(pigeon);
        }
    }*/

    let coop = &mut game_state.coops[0];
    if let Some(mut pigeon) = Coop::update_mouse_release(coop) {
        pigeon.trajectory = Some(game_state.aim_trajectory.clone());
        game_state.pigeons.push(pigeon);
    }

    for mut bubble in game_state.bubbles.iter_mut() {
        SpeechBubble::update_mouse_release(bubble, Point::new(mouse[0] as f32, mouse[1] as f32));
    }

    game_state.selected_coop = None;
}

fn on_game_over(game_state: &mut GameState) {
    // Test with toggle
    game_state.game_over = !game_state.game_over;
}

fn on_load(assets: &mut Assets, game_state: &mut GameState) {
    let pos_coop = geometry::Point::new(0.0, -0.9);
    game_state.coops.push(Coop::new(pos_coop));
    game_state.system_hubs.init();

    let pos_bubble = geometry::Point::new(0.2, 0.5);
    game_state.bubbles.push(SpeechBubble::new(pos_bubble,Size::new(0.4, 0.1), play_pigeon_sound, pos_bubble));

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
    // Rotate 2 radians per second.
    //self.game_state.rotation += 2.0 * args.dt;

    // Radioactive decay
    game_state.irradiance_field.decay(0.998f32);

    // Fixed radiation source for the reactor or whatever
    game_state.irradiance_field.splat(pos_to_irradiance_coord(Point::new(-0.5f32, 0.5f32)), 7f32, RadiationBlendMode::Max);

    let mut pigeon_to_nuke = None;
    for i in 0..game_state.pigeons.len() {
        let mut pigeon = &mut game_state.pigeons[i];
        if let PigeonStatus::ReachedDestination = pigeon.update((1.0 * args.dt) as f32) {
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

    game_state.pigeon_timer += args.dt;
    game_state.system_hubs.update_systems(args);
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
    let x: u32 = rng.gen_range(1,13);
    let s: String = x.to_string();
    let ss: &str = &s;

    let mut sound_file = String::from("assets/coo");
    sound_file.push_str(ss);
    sound_file.push_str(".wav");
    play_sound(&sound_file);
}

fn render_irradiance(
    factory: &mut gfx_device_gl::Factory,
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {

    let sf = &game_state.irradiance_field;
    let sf_texture = Texture::from_image(
        factory,
        &sf.to_image_buffer(),
        &TextureSettings::new()
    ).unwrap();

    let scale_0_to_1 = graphics::math::identity().trans(-1.0, -1.0).scale(2.0, 2.0);
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

    use graphics::*;
    if let Some(coop) = game_state.selected_coop {
        let trajectory = &game_state.aim_trajectory;
        if trajectory.points.len() < 2 {
    		return;
    	}

        let col = if game_state.coops[coop].can_fire() {
            [0.1, 1.0, 0.3, 1.0]
        } else {
            [1.0, 0.0, 0.0, 1.0]
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

fn render_coop(
    assets: &Assets,
    game_state: &GameState,
    render_state: &mut RenderState,
    args: &RenderArgs) {
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

    game_state.system_hubs.render_systems(render_state, args);
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
        let scale_0_to_1 = graphics::math::identity().trans(-1.0, -1.0).scale(2.0, 2.0);
        let gui_transform = scale_0_to_1
            .flip_v()
            .trans(0.0, -1.0)
            .scale(1.0 / assets.game_over.get_width() as f64, 1.0 / assets.game_over.get_height() as f64);
        Image::new().draw(&assets.game_over, &render_state.c.draw_state, gui_transform, render_state.g);
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
    let output_color = Typed::new(output_color);
    //let output_stencil = Typed::new(output_stencil);

    let (oscreen_tex, oscreen_srv, oscreen_rtv) = factory.create_render_target::<Srgba8>(
        draw_size.width as u16,
        draw_size.height as u16).unwrap();

    let (oscreen2_tex, oscreen2_srv, oscreen2_dsv) = factory.create_depth_stencil::<DepthStencil>(
        draw_size.width as u16,
        draw_size.height as u16).unwrap();

    /*fn create_render_target<T: RenderFormat + TextureFormat>(
        &mut self,
        width: Size,
        height: Size
    ) -> Result<(Texture<R, T::Surface>, ShaderResourceView<R, T::View>, RenderTargetView<R, T>), CombinedError> { ... }

*/

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRI_VERTS, ());

	let pso = factory.create_pipeline_simple(
		include_bytes!("../assets/Yasqueen.glslv"),
		include_bytes!("../assets/Yasqueen.glslf"),
		pipe::new()).unwrap();

	let mut data = pipe::Data {
		vbuf: vertex_buffer,
		out: output_color,
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
        selected_coop: None,
        pigeon_timer: 0.0,
    };

    on_load(&mut assets, &mut game_state);

    let mut cursor = Point::new(0f32, 0f32);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            //g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
            g2d.draw(&mut encoder, &oscreen_rtv, &oscreen2_dsv, args.viewport(), |c, g| {
                let mut render_state = RenderState { g, c };
                clear([0.0, 0.0, 0.0, 1.0], render_state.g);
                render_irradiance(&mut factory, &assets, &game_state, &mut render_state, &args);
                render_trajectory(&assets, &game_state, &mut render_state, &args);
                render_pigeons(&assets, &game_state, &mut render_state, &args);
                render_coop(&assets, &game_state, &mut render_state, &args);
                render_hubs(&assets, &game_state, &mut render_state, &args);
                render_ui(&assets, &game_state, &mut render_state, &args);

                text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw("IRRADIANT DESCENT",
                    &mut glyph_cache,
                    &DrawState::default(),
                    render_state.c.transform.trans(10.0, 100.0), render_state.g).unwrap();
            });

            /*let img_info = ImageInfoCommon {
                xoffset: 0,
                yoffset: 0,
                zoffset: 0,
                width: draw_size.width,
                height: draw_size.height,
                depth: 1,
                format: Srgba8,
                mipmap: 0 };

            let copy_src = TextureCopyRegion {
                texture: oscreen_tex,
                kind: Kind::D2,
                None,
                info: img_info };

            let copy_dst = TextureCopyRegion {
                texture: output_color,
                kind: Kind::D2,
                None,
                info: img_info };

            encoder.copy_texture_to_texture()*/
            encoder.clear(&data.out, [1.0, 0.0, 0.0, 1.0]);
            encoder.draw(&slice, &pso, &data);
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
            on_mouse_click(&mut game_state, [cursor.x as f64, cursor.y as f64]);
        }

        if let Some(Button::Mouse(_)) = e.release_args() {
            on_mouse_release(&mut game_state, [cursor.x as f64, cursor.y as f64]);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::C {
                play_pigeon_sound();
            }

            if key == Key::G {
                on_game_over(&mut game_state);
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

        e.text(|text| {
            println!("Typed '{}'", text)
        });

        e.resize(|w, h| {
            println!("Resized '{}, {}'", w, h)
        });
    }
}