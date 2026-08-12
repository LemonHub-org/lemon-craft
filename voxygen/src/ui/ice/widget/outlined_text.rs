//! Text with an outline (port of the Conrod `OutlinedText`).
use crate::ui::ice::{FontId, IcedRenderer, renderer::Primitive};
use iced::{
    Color, Element, Hasher, HorizontalAlignment, Layout, Length, Point, Rectangle, Size,
    VerticalAlignment, Widget, layout, mouse, text,
};
use std::hash::Hash;

/// A paragraph of text drawn four times offset by `outline_width` behind the
/// base text to produce an outline effect.
pub struct OutlinedText {
    content: String,
    color: Option<Color>,
    outline_color: Option<Color>,
    outline_width: f32,
    font: FontId,
    font_size: u16,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl OutlinedText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: None,
            outline_color: None,
            outline_width: 0.0,
            font: FontId::default(),
            font_size: 20,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    #[must_use]
    pub fn outline_color(mut self, color: Color) -> Self {
        self.outline_color = Some(color);
        self
    }

    #[must_use]
    pub fn outline_width(mut self, outline_width: f32) -> Self {
        self.outline_width = outline_width;
        self
    }

    #[must_use]
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    #[must_use]
    pub fn font_size(mut self, font_size: u16) -> Self {
        self.font_size = font_size;
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

    #[must_use]
    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    #[must_use]
    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<M> Widget<M, IcedRenderer> for OutlinedText {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        use text::Renderer as _;

        let limits = limits.width(self.width).height(self.height);
        let bounds = limits.max();

        let (width, height) = renderer.measure(&self.content, self.font_size, self.font, bounds);

        let size = limits.resolve(Size::new(width, height));
        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();
        let mut primitives = Vec::new();

        // Outline copies.
        if self.outline_width > 0.0 {
            let shift = self.outline_width;
            for (dx, dy) in [
                (shift, shift),
                (-shift, shift),
                (shift, -shift),
                (-shift, -shift),
            ] {
                let (primitive, _) = <IcedRenderer as iced::text::Renderer>::draw(
                    renderer,
                    defaults,
                    Rectangle {
                        x: bounds.x + dx,
                        y: bounds.y + dy,
                        ..bounds
                    },
                    &self.content,
                    self.font_size,
                    self.font,
                    self.outline_color,
                    self.horizontal_alignment,
                    self.vertical_alignment,
                );
                primitives.push(primitive);
            }
        }

        // Base text.
        let (primitive, _) = <IcedRenderer as iced::text::Renderer>::draw(
            renderer,
            defaults,
            bounds,
            &self.content,
            self.font_size,
            self.font,
            self.color,
            self.horizontal_alignment,
            self.vertical_alignment,
        );
        primitives.push(primitive);

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash(state);
        self.font_size.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<'a, M> From<OutlinedText> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(text: OutlinedText) -> Element<'a, M, IcedRenderer> { Element::new(text) }
}
