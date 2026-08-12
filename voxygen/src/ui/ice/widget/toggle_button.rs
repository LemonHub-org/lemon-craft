//! Two-state image button (port of the Conrod `ToggleButton`).
use super::image;
use crate::ui::ice::{IcedRenderer, renderer::Primitive};
use iced::{
    Element, Event, Hasher, Layout, Length, Point, Rectangle, Size, Widget, event, layout, mouse,
};
use std::hash::Hash;
use vek::*;

/// Image states of a button: default + optional hover/press variants.
#[derive(Debug, Clone, Copy)]
pub struct ImageStates {
    pub default: image::Handle,
    pub hover: Option<image::Handle>,
    pub press: Option<image::Handle>,
    pub color: Option<Rgba<u8>>,
}

impl ImageStates {
    pub fn new(default: image::Handle) -> Self {
        Self {
            default,
            hover: None,
            press: None,
            color: None,
        }
    }

    #[must_use]
    pub fn hover(mut self, hover: image::Handle) -> Self {
        self.hover = Some(hover);
        self
    }

    #[must_use]
    pub fn press(mut self, press: image::Handle) -> Self {
        self.press = Some(press);
        self
    }

    #[must_use]
    pub fn color(mut self, color: Rgba<u8>) -> Self {
        self.color = Some(color);
        self
    }
}

/// Per-widget state for press/hover tracking.
#[derive(Debug, Default)]
pub struct State {
    hover: bool,
    pressed: bool,
}

/// A button that toggles between two images and emits the new value when
/// clicked.
pub struct ToggleButton<'a, M> {
    value: bool,
    f_image: ImageStates,
    t_image: ImageStates,
    on_toggle: Box<dyn Fn(bool) -> M + 'a>,
    width: Length,
    height: Length,
    state: &'a mut State,
}

impl<'a, M> ToggleButton<'a, M> {
    pub fn new(
        value: bool,
        f_image: ImageStates,
        t_image: ImageStates,
        on_toggle: impl Fn(bool) -> M + 'a,
        state: &'a mut State,
    ) -> Self {
        Self {
            value,
            f_image,
            t_image,
            on_toggle: Box::new(on_toggle),
            width: Length::Shrink,
            height: Length::Shrink,
            state,
        }
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<M> Widget<M, IcedRenderer> for ToggleButton<'_, M> {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, _renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        _renderer: &mut IcedRenderer,
        _defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();
        let image = if self.value {
            &self.t_image
        } else {
            &self.f_image
        };
        let handle = if self.state.pressed {
            image.press.or(image.hover).unwrap_or(image.default)
        } else if bounds.contains(cursor_position) {
            image.hover.unwrap_or(image.default)
        } else {
            image.default
        };
        let color = image.color.unwrap_or(Rgba::broadcast(255));
        (
            Primitive::Image {
                handle: (handle, crate::ui::graphic::Rotation::None),
                bounds,
                color,
                source_rect: None,
            },
            if bounds.contains(cursor_position) {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn iced::Clipboard,
        messages: &mut Vec<M>,
    ) -> event::Status {
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.hover = bounds.contains(cursor_position);
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if bounds.contains(cursor_position) {
                    self.state.pressed = true;
                }
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(new_value) = toggle_on_release(
                    self.state.pressed,
                    bounds.contains(cursor_position),
                    self.value,
                ) {
                    self.state.pressed = false;
                    self.value = new_value;
                    messages.push((self.on_toggle)(self.value));
                } else {
                    self.state.pressed = false;
                }
            },
            _ => {},
        }
        event::Status::Ignored
    }
}

/// Pure click detection for a toggle: a click is a press followed by a release
/// within the widget bounds.
fn toggle_on_release(pressed: bool, released_in_bounds: bool, value: bool) -> Option<bool> {
    (pressed && released_in_bounds).then_some(!value)
}

impl<'a, M> From<ToggleButton<'a, M>> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(button: ToggleButton<'a, M>) -> Element<'a, M, IcedRenderer> { Element::new(button) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_in_bounds_toggles() {
        assert_eq!(toggle_on_release(true, true, false), Some(true));
        assert_eq!(toggle_on_release(true, true, true), Some(false));
    }

    #[test]
    fn release_outside_bounds_does_not_toggle() {
        assert_eq!(toggle_on_release(true, false, false), None);
        assert_eq!(toggle_on_release(false, true, false), None);
    }
}
