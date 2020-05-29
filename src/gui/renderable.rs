use imgui::*;

pub mod psola;
pub use psola::*;

pub trait Renderable {
    fn render(&mut self, ui: &Ui, position: [f32; 2]);
}

