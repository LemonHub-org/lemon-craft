//! 9-slice image frame widget (port of the Conrod `ImageFrame`).
use super::image;
use crate::ui::ice::{IcedRenderer, renderer::Primitive};
use iced::{Color, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget, layout, mouse};
use std::hash::Hash;
use vek::*;

/// Center of the frame: a plain color or an image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Center {
    Plain(Color),
    Image(image::Handle),
}

impl From<Color> for Center {
    fn from(color: Color) -> Self { Center::Plain(color) }
}

impl From<image::Handle> for Center {
    fn from(image: image::Handle) -> Self { Center::Image(image) }
}

/// Border thickness for each side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderSize {
    pub top: f32,
    pub bottom: f32,
    pub right: f32,
    pub left: f32,
}

impl From<f32> for BorderSize {
    fn from(thickness: f32) -> Self {
        Self {
            top: thickness,
            bottom: thickness,
            right: thickness,
            left: thickness,
        }
    }
}

impl From<[f32; 2]> for BorderSize {
    fn from([vertical, horizontal]: [f32; 2]) -> Self {
        Self {
            top: horizontal,
            bottom: horizontal,
            right: vertical,
            left: vertical,
        }
    }
}

impl From<[f32; 4]> for BorderSize {
    fn from(vals: [f32; 4]) -> Self {
        Self {
            top: vals[0],
            bottom: vals[1],
            right: vals[2],
            left: vals[3],
        }
    }
}

/// Computes the 9-slice rectangles of a frame.
///
/// Slice order: [right, top-right, top, top-left, left, bottom-left, bottom,
/// bottom-right, center].
pub fn nine_slice_bounds(bounds: Rectangle, border: f32) -> [Option<Rectangle>; 9] {
    let t = border.min(bounds.height);
    let b = border.min(bounds.height);
    let r = border.min(bounds.width);
    let l = border.min(bounds.width);
    let inner_width = (bounds.width - r - l).max(0.0);
    let inner_height = (bounds.height - t - b).max(0.0);
    // Right-side slices have no room once the borders overlap the whole
    // width; clamp them to the inner width.
    let right_w = r.min(inner_width);

    let rect = |x, y, w, h| {
        if w <= 0.0 || h <= 0.0 {
            None
        } else {
            Some(Rectangle {
                x,
                y,
                width: w,
                height: h,
            })
        }
    };

    let right_x = bounds.x + l + inner_width;
    let right = rect(right_x, bounds.y, right_w, inner_height);
    let tr = rect(right_x, bounds.y, right_w, t);
    let top = rect(bounds.x + l, bounds.y, inner_width, t);
    let tl = rect(bounds.x, bounds.y, l, t);
    let left = rect(bounds.x, bounds.y + t, l, inner_height);
    let bl = rect(bounds.x, bounds.y + t + inner_height, l, b);
    let bottom = rect(bounds.x + l, bounds.y + t + inner_height, inner_width, b);
    let br = rect(right_x, bounds.y + t + inner_height, right_w, b);
    let center = rect(bounds.x + l, bounds.y + t, inner_width, inner_height);

    [right, tr, top, tl, left, bl, bottom, br, center]
}

/// A widget that draws an image frame: 4 edge images, 4 corner images and a
/// center fill.
pub struct ImageFrame {
    // Edge images [top, bottom, right, left].
    edges: [image::Handle; 4],
    // Corner images [tr, tl, br, bl].
    corners: [image::Handle; 4],
    center: Center,
    border_size: BorderSize,
    // Color applied to all images making up the frame.
    color: Option<Rgba<u8>>,
    width: Length,
    height: Length,
}

impl ImageFrame {
    pub fn new<C: Into<Center>, B: Into<BorderSize>>(
        edges: [image::Handle; 4],
        corners: [image::Handle; 4],
        center: C,
        border_size: B,
    ) -> Self {
        Self {
            edges,
            corners,
            center: center.into(),
            border_size: border_size.into(),
            color: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    #[must_use]
    pub fn color(mut self, color: Rgba<u8>) -> Self {
        self.color = Some(color);
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
}

impl<M> Widget<M, IcedRenderer> for ImageFrame {
    fn width(&self) -> Length { self.width }

    fn height(&self) -> Length { self.height }

    fn layout(&self, _renderer: &IcedRenderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        _defaults: &<IcedRenderer as iced::Renderer>::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> <IcedRenderer as iced::Renderer>::Output {
        let bounds = layout.bounds();
        let mut primitives = Vec::new();

        let border = self.border_size;
        let t = border.top.min(bounds.height);
        let b = border.bottom.min(bounds.height);
        let r = border.right.min(bounds.width);
        let l = border.left.min(bounds.width);
        let inner_width = (bounds.width - r - l).max(0.0);
        let inner_height = (bounds.height - t - b).max(0.0);
        let right_w = r.min(inner_width);
        let right_x = bounds.x + l + inner_width;

        let color = self.color.unwrap_or(Rgba::broadcast(255));
        let mut set_image = |handle, rect: Rectangle, primitives: &mut Vec<Primitive>| {
            if rect.width > 0.0 && rect.height > 0.0 {
                primitives.push(renderer.draw_image(handle, rect, color));
            }
        };

        // Right edge.
        set_image(
            self.edges[2],
            Rectangle {
                x: right_x,
                y: bounds.y,
                width: right_w,
                height: inner_height,
            },
            &mut primitives,
        );
        // Top-right corner.
        set_image(
            self.corners[0],
            Rectangle {
                x: right_x,
                y: bounds.y,
                width: right_w,
                height: t,
            },
            &mut primitives,
        );
        // Top edge.
        set_image(
            self.edges[0],
            Rectangle {
                x: bounds.x + l,
                y: bounds.y,
                width: inner_width,
                height: t,
            },
            &mut primitives,
        );
        // Top-left corner.
        set_image(
            self.corners[1],
            Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: l,
                height: t,
            },
            &mut primitives,
        );
        // Left edge.
        set_image(
            self.edges[3],
            Rectangle {
                x: bounds.x,
                y: bounds.y + t,
                width: l,
                height: inner_height,
            },
            &mut primitives,
        );
        // Bottom-left corner.
        set_image(
            self.corners[3],
            Rectangle {
                x: bounds.x,
                y: bounds.y + t + inner_height,
                width: l,
                height: b,
            },
            &mut primitives,
        );
        // Bottom edge.
        set_image(
            self.edges[1],
            Rectangle {
                x: bounds.x + l,
                y: bounds.y + t + inner_height,
                width: inner_width,
                height: b,
            },
            &mut primitives,
        );
        // Bottom-right corner.
        set_image(
            self.corners[2],
            Rectangle {
                x: right_x,
                y: bounds.y + t + inner_height,
                width: right_w,
                height: b,
            },
            &mut primitives,
        );

        // Center.
        let center_bounds = Rectangle {
            x: bounds.x + l,
            y: bounds.y + t,
            width: inner_width,
            height: inner_height,
        };
        match self.center {
            Center::Plain(color) => {
                if center_bounds.width > 0.0 && center_bounds.height > 0.0 {
                    primitives
                        .push(renderer.draw_rectangle(center_bounds, color.into_linear().into()));
                }
            },
            Center::Image(handle) => {
                set_image(handle, center_bounds, &mut primitives);
            },
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<'a, M> From<ImageFrame> for Element<'a, M, IcedRenderer>
where
    M: 'a,
{
    fn from(frame: ImageFrame) -> Element<'a, M, IcedRenderer> { Element::new(frame) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_slice_geometry() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 60.0,
        };
        let slices = nine_slice_bounds(bounds, 10.0);
        // Right edge.
        let right = slices[0].unwrap();
        assert_eq!(right.x, 90.0);
        assert_eq!(right.width, 10.0);
        assert_eq!(right.height, 40.0);
        // Top edge.
        let top = slices[2].unwrap();
        assert_eq!(top.y, 0.0);
        assert_eq!(top.height, 10.0);
        assert_eq!(top.width, 80.0);
        // Center.
        let center = slices[8].unwrap();
        assert_eq!(center.x, 10.0);
        assert_eq!(center.y, 10.0);
        assert_eq!(center.width, 80.0);
        assert_eq!(center.height, 40.0);
    }

    #[test]
    fn nine_slice_clamps_to_small_bounds() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 5.0,
            height: 60.0,
        };
        let slices = nine_slice_bounds(bounds, 10.0);
        assert!(slices[0].is_none(), "right edge clamped to zero width");
        assert!(slices[8].is_none(), "center clamped to zero width");
    }

    #[test]
    fn border_size_conversions() {
        let b: BorderSize = 5.0.into();
        assert_eq!(b.top, 5.0);
        assert_eq!(b.right, 5.0);
        let b: BorderSize = [1.0, 2.0].into();
        assert_eq!(b.top, 2.0);
        assert_eq!(b.right, 1.0);
        let b: BorderSize = [1.0, 2.0, 3.0, 4.0].into();
        assert_eq!(b.top, 1.0);
        assert_eq!(b.bottom, 2.0);
        assert_eq!(b.right, 3.0);
        assert_eq!(b.left, 4.0);
    }
}
