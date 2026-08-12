//! Vertical list of radio buttons with labels (port of the Conrod
//! `RadioList`).
use super::image;
use crate::ui::ice::{FontId, IcedRenderer, renderer::Primitive};
use iced::{
    Color, Element, Event, Hasher, HorizontalAlignment, Layout, Length, Point, Rectangle, Size,
    VerticalAlignment, Widget, event, layout, mouse,
};
use std::hash::Hash;
use vek::*;

/// Per-widget state for hover tracking.
#[derive(Debug, Default)]
pub struct State {
    hovered_row: Option<usize>,
}

/// Computes the bounds of a row for hit testing.
pub fn row_bounds(rect: Rectangle, row: usize, button_dims: Vec2<f32>, spacing: f32) -> Rectangle {
    let label_space = rect.width - button_dims.x;
    Rectangle {
        x: rect.x,
        y: rect.y + row as f32 * (button_dims.y + spacing),
        width: button_dims.x + label_space,
        height: button_dims.y,
    }
}

/// A vertical list of radio buttons, each with an optional label.
pub struct RadioList<'a, T, M> {
    f_image: image::Handle,
    t_image: image::Handle,
    selected: usize,
    options_labels: &'a [(&'a T, &'a str)],
    label_color: Option<Color>,
    label_font: FontId,
    label_font_size: u16,
    label_spacing: f32,
    button_spacing: f32,
    button_dims: Vec2<f32>,
    on_change: Box<dyn Fn(usize) -> M + 'a>,
    width: Length,
    height: Length,
    state: &'a mut State,
    _option: std::marker::PhantomData<&'a T>,
}

impl<'a, T, M> RadioList<'a, T, M> {
    pub fn new(
        selected: usize,
        f_image: image::Handle,
        t_image: image::Handle,
        options_labels: &'a [(&'a T, &'a str)],
        on_change: impl Fn(usize) -> M + 'a,
        state: &'a mut State,
    ) -> Self {
        Self {
            f_image,
            t_image,
            selected,
            options_labels,
            label_color: None,
            label_font: FontId::default(),
            label_font_size: 20,
            label_spacing: 10.0,
            button_spacing: 5.0,
            button_dims: Vec2::new(15.0, 15.0),
            on_change: Box::new(on_change),
            width: Length::Shrink,
            height: Length::Shrink,
            state,
            _option: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }

    #[must_use]
    pub fn font(mut self, font: FontId) -> Self {
        self.label_font = font;
        self
    }

    #[must_use]
    pub fn font_size(mut self, font_size: u16) -> Self {
        self.label_font_size = font_size;
        self
    }

    #[must_use]
    pub fn label_spacing(mut self, spacing: f32) -> Self {
        self.label_spacing = spacing;
        self
    }

    #[must_use]
    pub fn button_spacing(mut self, spacing: f32) -> Self {
        self.button_spacing = spacing;
        self
    }

    #[must_use]
    pub fn button_dims(mut self, dims: Vec2<f32>) -> Self {
        self.button_dims = dims;
        self
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

    fn row_count(&self) -> usize { self.options_labels.len() }
}

impl<T, M> Widget<M, IcedRenderer> for RadioList<'_, T, M> {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        use iced::text::Renderer as _;

        let limits = limits.width(self.width).height(self.height);
        let label_width = self
            .options_labels
            .iter()
            .map(|(_, label)| {
                let (w, _) = renderer.measure(
                    label,
                    self.label_font_size,
                    self.label_font,
                    Size::new(f32::INFINITY, f32::INFINITY),
                );
                w
            })
            .fold(0.0, f32::max);
        let width = self.button_dims.x + self.label_spacing + label_width;
        let height = self.row_count() as f32 * (self.button_dims.y + self.button_spacing);

        let size = limits.resolve(Size::new(width, height));
        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        use iced::text::Renderer as _;

        let bounds = layout.bounds();
        let mut primitives = Vec::new();

        for (i, (_, label)) in self.options_labels.iter().enumerate() {
            let button_rect = row_bounds(bounds, i, self.button_dims, self.button_spacing);
            let handle = if i == self.selected {
                self.t_image
            } else {
                self.f_image
            };
            primitives.push(renderer.draw_image(
                handle,
                Rectangle {
                    x: button_rect.x,
                    y: button_rect.y,
                    width: self.button_dims.x,
                    height: self.button_dims.y,
                },
                Rgba::broadcast(255),
            ));

            let (text_width, _) = renderer.measure(
                label,
                self.label_font_size,
                self.label_font,
                Size::new(f32::INFINITY, f32::INFINITY),
            );
            let (primitive, _) = <IcedRenderer as iced::text::Renderer>::draw(
                renderer,
                defaults,
                Rectangle {
                    x: button_rect.x + self.button_dims.x + self.label_spacing,
                    y: button_rect.y,
                    width: text_width,
                    height: self.button_dims.y,
                },
                label,
                self.label_font_size,
                self.label_font,
                self.label_color,
                HorizontalAlignment::Left,
                VerticalAlignment::Center,
            );
            primitives.push(primitive);
        }

        let mouse_interaction = if self.state.hovered_row.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        };

        let _ = cursor_position;
        (Primitive::Group { primitives }, mouse_interaction)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.button_dims.x.to_bits().hash(state);
        self.button_dims.y.to_bits().hash(state);
        self.options_labels.len().hash(state);
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
        let row_at = |cursor: Point| {
            (0..self.row_count()).find(|&i| {
                row_bounds(bounds, i, self.button_dims, self.button_spacing).contains(cursor)
            })
        };

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.hovered_row = row_at(cursor_position);
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(row) = row_at(cursor_position)
                    && row != self.selected
                {
                    self.selected = row;
                    messages.push((self.on_change)(row));
                }
            },
            _ => {},
        }
        event::Status::Ignored
    }
}

impl<'a, T, M> From<RadioList<'a, T, M>> for Element<'a, M, IcedRenderer>
where
    T: 'a,
    M: 'a,
{
    fn from(list: RadioList<'a, T, M>) -> Element<'a, M, IcedRenderer> { Element::new(list) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_bounds_layout() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        };
        let dims = Vec2::new(15.0, 15.0);
        let row0 = row_bounds(rect, 0, dims, 5.0);
        assert_eq!(row0.y, 0.0);
        let row1 = row_bounds(rect, 1, dims, 5.0);
        assert_eq!(row1.y, 20.0);
        let row2 = row_bounds(rect, 2, dims, 5.0);
        assert_eq!(row2.y, 40.0);
        assert_eq!(
            row2.width, 200.0,
            "rows span the full width for hit testing"
        );
    }

    #[test]
    fn row_bounds_containment() {
        let rect = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 200.0,
            height: 100.0,
        };
        let dims = Vec2::new(15.0, 15.0);
        assert!(row_bounds(rect, 0, dims, 5.0).contains(Point { x: 20.0, y: 20.0 }));
        assert!(!row_bounds(rect, 1, dims, 5.0).contains(Point { x: 20.0, y: 20.0 }));
        assert!(row_bounds(rect, 1, dims, 5.0).contains(Point { x: 20.0, y: 40.0 }));
    }
}
