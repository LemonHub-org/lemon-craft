use super::super::super::widget::image;
use crate::ui::theme::{alpha, brand, to_rgba_u8};
use vek::Rgba;

/// Container Border
#[derive(Clone, Copy)]
pub enum Border {
    DoubleCornerless {
        inner: Rgba<u8>,
        outer: Rgba<u8>,
    },
    Image {
        corner: image::Handle,
        edge: image::Handle,
    },
    None,
}

/// Background of the container
#[derive(Clone, Copy, Default)]
pub enum Style {
    Image(image::Handle, Rgba<u8>),
    Color(Rgba<u8>, Border),
    #[default]
    None,
}

impl Style {
    /// Shorthand for common case where the color of the image is not modified
    pub fn image(image: image::Handle) -> Self { Self::Image(image, Rgba::broadcast(255)) }

    /// Shorthand for a color background with no border
    pub fn color(color: Rgba<u8>) -> Self { Self::Color(color, Border::None) }

    /// Standard light menu surface without a border.
    pub fn panel() -> Self { Self::color(to_rgba_u8(brand::PANEL_BG)) }

    /// Alternate light menu surface for secondary/pressed regions.
    pub fn panel_alt() -> Self { Self::color(to_rgba_u8(brand::PANEL_BG_ALT)) }

    /// Standard menu surface with the shared Lemon Fresh frame treatment.
    pub fn panel_with_frame() -> Self {
        Self::color_with_double_cornerless_border(
            to_rgba_u8(brand::PANEL_BG),
            to_rgba_u8(brand::PANEL_FILL),
            to_rgba_u8(brand::FRAME),
        )
    }

    /// Light translucent surface used over menu artwork.
    pub fn panel_overlay() -> Self { Self::color(to_rgba_u8(alpha(brand::PANEL_BG, 0.9))) }

    /// Shorthand for a color background with a cornerless border
    pub fn color_with_double_cornerless_border(
        color: Rgba<u8>,
        inner: Rgba<u8>,
        outer: Rgba<u8>,
    ) -> Self {
        Self::Color(color, Border::DoubleCornerless { inner, outer })
    }

    /// Shorthand for a color background with image borders where the corners
    /// are inset
    pub fn color_with_image_border(
        color: Rgba<u8>,
        corner: image::Handle,
        edge: image::Handle,
    ) -> Self {
        Self::Color(color, Border::Image { corner, edge })
    }
}
