//! Text with inline image icons (port of the Conrod `RichText`).
//!
//! `RichText` parses strings for tags (e.g. `:south:`) and replaces them with
//! the corresponding icons resolved through the provided resolver.
use super::image;
use crate::ui::ice::{FontId, IcedRenderer, renderer::Primitive};
use iced::{
    Color, Element, Hasher, HorizontalAlignment, Layout, Length, Point, Rectangle, Size,
    VerticalAlignment, Widget, layout, mouse,
};
use regex::Regex;
use std::{hash::Hash, sync::LazyLock};
use vek::*;

// Represents a piece of the rich text flow.
#[derive(Debug, Clone, PartialEq)]
enum TextSegment<'a, H> {
    Text(&'a str),
    // Font size is [w, h].
    Image(H),
    Newline,
}

/// A widget for rendering text with inline images/icons.
pub struct RichText<'a, M> {
    segments: Vec<TextSegment<'a, image::Handle>>,
    color: Option<Color>,
    font: FontId,
    font_size: u16,
    line_spacing: f32,
    justify: HorizontalAlignment,
    width: Length,
    height: Length,
    // Kept for the message type in the `Widget` impl.
    _message: std::marker::PhantomData<M>,
}

impl<'a, M> RichText<'a, M> {
    /// Creates a new `RichText` widget.
    ///
    /// # Arguments
    /// * `string` - the text to display. Use tags like `:name:` to insert
    ///   images.
    /// * `resolver` - maps a tag to an image handle.
    pub fn new(string: &'a str, resolver: impl Fn(&str) -> Option<image::Handle>) -> Self {
        Self {
            segments: parse(string, resolver),
            color: None,
            font: FontId::default(),
            font_size: 20,
            line_spacing: 5.0,
            justify: HorizontalAlignment::Left,
            width: Length::Shrink,
            height: Length::Shrink,
            _message: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
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
    pub fn line_spacing(mut self, line_spacing: f32) -> Self {
        self.line_spacing = line_spacing;
        self
    }

    #[must_use]
    pub fn justify(mut self, justify: HorizontalAlignment) -> Self {
        self.justify = justify;
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

    /// The number of lines in the parsed content.
    pub fn line_count(&self) -> usize {
        self.segments
            .iter()
            .filter(|s| matches!(s, TextSegment::Newline))
            .count()
            + 1
    }

    /// Line widths in the given renderer (for justify calculations).
    fn line_widths(&self, renderer: &IcedRenderer) -> Vec<f32> {
        use iced::text::Renderer as _;
        let mut widths = Vec::new();
        let mut current = 0.0;
        for segment in &self.segments {
            match segment {
                TextSegment::Text(s) => {
                    let (w, _) = renderer.measure(
                        s,
                        self.font_size,
                        self.font,
                        Size::new(f32::INFINITY, f32::INFINITY),
                    );
                    current += w;
                },
                TextSegment::Image(_) => current += self.font_size as f32,
                TextSegment::Newline => {
                    widths.push(current);
                    current = 0.0;
                },
            }
        }
        widths.push(current);
        widths
    }

    /// Total size of the parsed content in the given renderer.
    fn content_size(&self, renderer: &IcedRenderer) -> Size {
        use iced::text::Renderer as _;
        let line_height = self.font_size as f32 + self.line_spacing;
        let mut max_w: f32 = 0.0;
        let mut current_w: f32 = 0.0;
        let mut total_h = line_height;
        for segment in &self.segments {
            match segment {
                TextSegment::Text(s) => {
                    let (w, _) = renderer.measure(
                        s,
                        self.font_size,
                        self.font,
                        Size::new(f32::INFINITY, f32::INFINITY),
                    );
                    current_w += w;
                },
                TextSegment::Image(_) => current_w += self.font_size as f32,
                TextSegment::Newline => {
                    max_w = max_w.max(current_w);
                    current_w = 0.0;
                    total_h += line_height;
                },
            }
        }
        Size::new(max_w.max(current_w), total_h)
    }
}

impl<M> Widget<M, IcedRenderer> for RichText<'_, M> {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = self.content_size(renderer);
        let size = limits.resolve(size);
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
        use iced::text::Renderer as _;

        let bounds = layout.bounds();
        let line_height = self.font_size as f32 + self.line_spacing;
        let line_widths = self.line_widths(renderer);

        let mut primitives = Vec::new();
        let mut y_cursor = 0.0;
        let mut line_idx = 0;
        let mut x_cursor = match self.justify {
            HorizontalAlignment::Left => 0.0,
            HorizontalAlignment::Right => bounds.width - line_widths[line_idx],
            HorizontalAlignment::Center => (bounds.width - line_widths[line_idx]) / 2.0,
        };

        let text_color = self.color.unwrap_or(defaults.text_color);

        for segment in &self.segments {
            match segment {
                TextSegment::Text(string) => {
                    if string.is_empty() {
                        continue;
                    }
                    let (text_width, _) = renderer.measure(
                        string,
                        self.font_size,
                        self.font,
                        Size::new(f32::INFINITY, f32::INFINITY),
                    );
                    let (primitive, _) = <IcedRenderer as iced::text::Renderer>::draw(
                        renderer,
                        defaults,
                        Rectangle {
                            x: bounds.x + x_cursor,
                            y: bounds.y + y_cursor,
                            width: text_width,
                            height: line_height,
                        },
                        string,
                        self.font_size,
                        self.font,
                        Some(text_color),
                        HorizontalAlignment::Left,
                        VerticalAlignment::Top,
                    );
                    primitives.push(primitive);

                    x_cursor += text_width;
                },
                TextSegment::Image(handle) => {
                    let image_size = self.font_size as f32;
                    // Vertical offset to visually align icons with the text
                    // baseline.
                    let v_offset = 1.5;
                    primitives.push(renderer.draw_image(
                        *handle,
                        Rectangle {
                            x: bounds.x + x_cursor,
                            y: bounds.y + y_cursor + v_offset,
                            width: image_size,
                            height: image_size,
                        },
                        // Opacity value is important: images inherit the text
                        // alpha.
                        Rgba::new(
                            (text_color.r * 255.0) as u8,
                            (text_color.g * 255.0) as u8,
                            (text_color.b * 255.0) as u8,
                            (text_color.a * 255.0) as u8,
                        ),
                    ));

                    x_cursor += image_size;
                },
                TextSegment::Newline => {
                    line_idx += 1;
                    x_cursor = match self.justify {
                        HorizontalAlignment::Left => 0.0,
                        HorizontalAlignment::Right => bounds.width - line_widths[line_idx],
                        HorizontalAlignment::Center => (bounds.width - line_widths[line_idx]) / 2.0,
                    };
                    y_cursor += line_height;
                },
            }
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.font_size.hash(state);
        std::hash::Hash::hash(&self.line_spacing.to_bits(), state);
        self.width.hash(state);
        self.height.hash(state);
        self.segments.len().hash(state);
    }
}

// Do a forward pass through the input to pre-process it.
fn parse<'a, H>(input: &'a str, resolver: impl Fn(&str) -> Option<H>) -> Vec<TextSegment<'a, H>> {
    // A magical incantation that splits strings by double colons (e.g.
    // ":icon:").
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r":(?P<tag>[^:\s]+):").expect("Invalid Regex"));
    let mut segments = Vec::new();
    let mut last_end = 0;

    for check in RE.captures_iter(input) {
        let whole = check.get(0).unwrap();
        let tag = &check["tag"];

        // Push the text before the icon.
        if whole.start() > last_end {
            push_text_segments(&input[last_end..whole.start()], &mut segments);
        }

        // Add the icon to the output if it resolves.
        if let Some(handle) = resolver(tag) {
            segments.push(TextSegment::Image(handle));
        } else {
            // Unknown tags are kept as literal text.
            push_text_segments(whole.as_str(), &mut segments);
        }

        last_end = whole.end();
    }

    // Push trailing text.
    if last_end < input.len() {
        push_text_segments(&input[last_end..], &mut segments);
    }

    segments
}

fn push_text_segments<'a, H>(text: &'a str, segments: &mut Vec<TextSegment<'a, H>>) {
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            segments.push(TextSegment::Newline);
        }
        if !line.is_empty() {
            segments.push(TextSegment::Text(line));
        }
    }
}

impl<'a, M> From<RichText<'a, M>> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(text: RichText<'a, M>) -> Element<'a, M, IcedRenderer> { Element::new(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver(tag: &str) -> Option<u8> {
        match tag {
            "a" | "b" => Some(tag.len() as u8),
            _ => None,
        }
    }

    fn segments_of(text: &str) -> Vec<TextSegment<'_, u8>> { parse(text, resolver) }

    #[test]
    fn parses_simple_text() {
        let segments = segments_of("hello world");
        assert_eq!(segments, vec![TextSegment::Text("hello world")]);
    }

    #[test]
    fn parses_icons_between_text() {
        let segments = segments_of("press :a: to jump");
        assert_eq!(segments, vec![
            TextSegment::Text("press "),
            TextSegment::Image(1),
            TextSegment::Text(" to jump"),
        ]);
    }

    #[test]
    fn parses_newlines() {
        let segments = segments_of("line1\nline2");
        assert_eq!(segments, vec![
            TextSegment::Text("line1"),
            TextSegment::Newline,
            TextSegment::Text("line2"),
        ]);
    }

    #[test]
    fn newline_after_icon() {
        let segments = segments_of(":a:\n:bar:");
        assert_eq!(segments, vec![
            TextSegment::Image(1),
            TextSegment::Newline,
            TextSegment::Text(":bar:"),
        ]);
    }

    #[test]
    fn empty_string_yields_no_segments() {
        assert!(segments_of("").is_empty());
    }

    #[test]
    fn unknown_tags_stay_literal() {
        let segments = segments_of("x :unknown: y");
        assert_eq!(segments, vec![
            TextSegment::Text("x "),
            TextSegment::Text(":unknown:"),
            TextSegment::Text(" y"),
        ]);
    }

    #[test]
    fn line_count() {
        let rich = RichText::<()>::new("a\nb\nc", |_| None);
        assert_eq!(rich.line_count(), 3);
        let rich = RichText::<()>::new("abc", |_| None);
        assert_eq!(rich.line_count(), 1);
    }
}
