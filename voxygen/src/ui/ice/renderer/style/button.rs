use super::super::super::widget::image;
use crate::ui::theme::{brand, to_iced, to_rgba_u8};
use iced::Color;
use vek::Rgba;

#[derive(Clone, Copy)]
struct Background {
    default: image::Handle,
    hover: image::Handle,
    press: image::Handle,
    color: Rgba<u8>,
}

impl Background {
    fn new(image: image::Handle) -> Self {
        Self {
            default: image,
            hover: image,
            press: image,
            color: Rgba::white(),
        }
    }
}
// TODO: consider a different place for this
// Note: for now all buttons have an image background
#[derive(Clone, Copy)]
pub struct Style {
    background: Option<Background>,
    enabled_text: Color,
    disabled_text: Color,
}

impl Style {
    /// Shared selection-frame treatment for lists and option groups.
    pub fn selection(
        image: image::Handle,
        hover_image: image::Handle,
        press_image: image::Handle,
        tint: Rgba<u8>,
    ) -> Self {
        Self::new(image)
            .hover_image(hover_image)
            .press_image(press_image)
            .image_color(tint)
    }

    /// Standard Lemon Fresh treatment for ordinary menu buttons.
    ///
    /// The image states remain swappable so the current asset set can be
    /// replaced incrementally, while all generic menu buttons share one
    /// theme tint in the meantime.
    pub fn lemon_fresh(
        image: image::Handle,
        hover_image: image::Handle,
        press_image: image::Handle,
    ) -> Self {
        Self::new(image)
            .hover_image(hover_image)
            .press_image(press_image)
            .image_color(to_rgba_u8(brand::BUTTON_IMAGE_TINT))
    }

    /// High-contrast treatment for the main menu over the bright scene art.
    ///
    /// The legacy button PNGs are intentionally retained as state masks, but
    /// their ivory artwork is multiplied by a dark olive theme tint here so
    /// the menu does not read as a collection of white cards.
    pub fn main_menu(
        image: image::Handle,
        hover_image: image::Handle,
        press_image: image::Handle,
    ) -> Self {
        Self::new(image)
            .hover_image(hover_image)
            .press_image(press_image)
            .image_color(to_rgba_u8(brand::MAIN_MENU_BUTTON_TINT))
    }

    pub fn new(image: image::Handle) -> Self {
        Self {
            background: Some(Background::new(image)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn hover_image(mut self, image: image::Handle) -> Self {
        self.background = Some(match self.background {
            Some(mut background) => {
                background.hover = image;
                background
            },
            None => Background::new(image),
        });
        self
    }

    #[must_use]
    pub fn press_image(mut self, image: image::Handle) -> Self {
        self.background = Some(match self.background {
            Some(mut background) => {
                background.press = image;
                background
            },
            None => Background::new(image),
        });
        self
    }

    // TODO: this needs to be refactored since the color isn't used if there is no
    // background
    #[must_use]
    pub fn image_color(mut self, color: Rgba<u8>) -> Self {
        if let Some(background) = &mut self.background {
            background.color = color;
        }
        self
    }

    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.enabled_text = color;
        self
    }

    #[must_use]
    pub fn disabled_text_color(mut self, color: Color) -> Self {
        self.disabled_text = color;
        self
    }

    pub fn disabled(&self) -> (Option<(image::Handle, Rgba<u8>)>, Color) {
        (
            self.background.as_ref().map(|b| (b.default, b.color)),
            self.disabled_text,
        )
    }

    pub fn pressed(&self) -> (Option<(image::Handle, Rgba<u8>)>, Color) {
        (
            self.background.as_ref().map(|b| (b.press, b.color)),
            self.enabled_text,
        )
    }

    pub fn hovered(&self) -> (Option<(image::Handle, Rgba<u8>)>, Color) {
        (
            self.background.as_ref().map(|b| (b.hover, b.color)),
            self.enabled_text,
        )
    }

    pub fn active(&self) -> (Option<(image::Handle, Rgba<u8>)>, Color) {
        (
            self.background.as_ref().map(|b| (b.default, b.color)),
            self.enabled_text,
        )
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            enabled_text: to_iced(brand::TEXT_PRIMARY),
            disabled_text: to_iced(brand::TEXT_DISABLED),
        }
    }
}
