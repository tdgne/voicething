use crate::audio::stream::psola::PsolaNode;
use crate::audio::stream::node::HasId;
use super::Renderable;
use imgui::*;

impl Renderable for PsolaNode {
    fn render(&mut self, ui: &Ui, position: [f32; 2]) {
        Window::new(&im_str!("TD-PSOLA {}", self.id()))
            .always_auto_resize(true)
            .position(position, Condition::FirstUseEver)
            .build(&ui, || {
                VerticalSlider::new(
                    im_str!("pitch"),
                    [30.0, 200.0],
                    std::ops::RangeInclusive::new(0.5, 2.0),
                )
                .display_format(im_str!("%0.2f"))
                .build(&ui, self.ratio_mut());
            });
    }
}
