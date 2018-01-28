extern crate graphics;
use models::selectable::SelectableRect;
use geometry::{Point,Size};

use piston::input::RenderArgs;
use RenderState;

#[derive(Clone)]
pub struct SpeechBubble {
    pub rect: SelectableRect, 
    pub tip_pos: Point
}

impl SpeechBubble {
    /// Create a SpeechBubble
    pub fn new(position: Point, size: Size, on_click: fn(), tip_pos: Point) -> SpeechBubble {
        SpeechBubble { rect: SelectableRect::new(position, size, on_click), tip_pos: tip_pos }
    }

    /// Clicked on?
    pub fn update_mouse_release(&mut self, mouse: Point) {
        self.rect.update_mouse_release(mouse);
    }

    /// Mouse hovering?
    pub fn update_mouse_move(&mut self, dt: f32, mouse: Point) {
        self.rect.update_mouse_move(dt,mouse);
    }

    pub fn render_bubble(&self, render_state: &mut RenderState, args: &RenderArgs) {
        const WHITE:  [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        //self.rect.render_rect(render_state,args,WHITE);
    }


    pub fn get_text(&self)-> String
    {
        let s = String::from("DUNKA DUNKA HAS STARTED!");
        return s;
    }
    pub fn get_point(&self) -> &Point{
        return &self.rect.position;
    }
}