// TODO: expose to user
pub struct Defaults {
    pub text_color: iced::Color,
}

impl Default for Defaults {
    fn default() -> Self {
        use crate::ui::theme::{brand, to_iced};
        Self {
            text_color: to_iced(brand::TEXT_PRIMARY),
        }
    }
}
