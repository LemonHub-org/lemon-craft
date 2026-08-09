use super::{FILL_FRAC_ONE, FILL_FRAC_TWO, Imgs, LoginInfo, Message, Showing, TEXT_COLOR};
use crate::ui::{
    fonts::IcedFonts as Fonts,
    ice::{Element, component::neat_button, style},
};

use i18n::{LanguageMetadata, Localization};
use iced::{
    Align, Button, Column, Container, Length, Row, Scrollable, Space, Text, TextInput, button,
    scrollable, text_input,
};
use vek::*;

const INPUT_WIDTH: u16 = 300;
const INPUT_TEXT_SIZE: u16 = 20;

/// Login screen for the main menu
#[derive(Default)]
pub struct Screen {
    quit_button: button::State,
    credits_button: button::State,
    language_select_button: button::State,

    error_okay_button: button::State,

    pub banner: LoginBanner,
    language_selection: LanguageSelectBanner,
}

impl Screen {
    pub(super) fn view(
        &mut self,
        fonts: &Fonts,
        imgs: &Imgs,
        login_info: &LoginInfo,
        error: Option<&str>,
        i18n: &Localization,
        show: &Showing,
        selected_language_index: Option<usize>,
        language_metadatas: &[LanguageMetadata],
        button_style: style::button::Style,
    ) -> Element<'_, Message> {
        let utility_buttons = Column::with_children(vec![
            neat_button(
                &mut self.language_select_button,
                i18n.get_msg("common-languages"),
                FILL_FRAC_ONE,
                button_style,
                Some(Message::OpenLanguageMenu),
            ),
            neat_button(
                &mut self.credits_button,
                i18n.get_msg("main-credits"),
                FILL_FRAC_ONE,
                button_style,
                Some(Message::ShowCredits),
            ),
            neat_button(
                &mut self.quit_button,
                i18n.get_msg("common-quit"),
                FILL_FRAC_ONE,
                button_style,
                Some(Message::Quit),
            ),
        ])
        .width(Length::Fill)
        .max_width(200)
        .spacing(4)
        .align_items(Align::Center)
        .into();

        let central_content = if let Some(error) = error {
            Container::new(
                Column::with_children(vec![
                    Container::new(Text::new(error).color(TEXT_COLOR).width(Length::Fill))
                        .height(Length::Fill)
                        .into(),
                    Container::new(neat_button(
                        &mut self.error_okay_button,
                        i18n.get_msg("common-okay"),
                        FILL_FRAC_ONE,
                        button_style,
                        Some(Message::CloseError),
                    ))
                    .width(Length::Fill)
                    .height(Length::Units(30))
                    .center_x()
                    .into(),
                ])
                .height(Length::Fill)
                .width(Length::Fill),
            )
            .width(Length::Units(400))
            .height(Length::Units(180))
            .padding(20)
            .into()
        } else {
            match show {
                Showing::Login => self.banner.view(fonts, login_info, i18n, button_style),
                Showing::Languages => self.language_selection.view(
                    fonts,
                    imgs,
                    i18n,
                    language_metadatas,
                    selected_language_index,
                    button_style,
                ),
            }
        };

        let central_panel = Column::with_children(vec![
            central_content,
            Space::new(Length::Fill, Length::Units(18)).into(),
            utility_buttons,
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(5)
        .align_items(Align::Center);

        let central_panel = Container::new(central_panel)
            .width(Length::Units(520))
            .height(Length::Units(420))
            .padding([24, 28]);

        let central_column = Container::new(central_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        central_column.into()
    }
}

#[derive(Default)]
pub struct LanguageSelectBanner {
    okay_button: button::State,
    language_buttons: Vec<button::State>,

    selection_list: scrollable::State,
}

impl LanguageSelectBanner {
    fn view(
        &mut self,
        fonts: &Fonts,
        imgs: &Imgs,
        i18n: &Localization,
        language_metadatas: &[LanguageMetadata],
        selected_language_index: Option<usize>,
        button_style: style::button::Style,
    ) -> Element<'_, Message> {
        // Reset button states if languages were added / removed
        if self.language_buttons.len() != language_metadatas.len() {
            self.language_buttons = vec![Default::default(); language_metadatas.len()];
        }

        let title = Text::new(i18n.get_msg("main-login-select_language"))
            .font(fonts.alkhemi.id)
            .color(TEXT_COLOR)
            .size(fonts.alkhemi.scale(30))
            .horizontal_alignment(iced::HorizontalAlignment::Center);

        let mut list = Scrollable::new(&mut self.selection_list)
            .spacing(8)
            .height(Length::Fill)
            .align_items(Align::Start);

        let list_items = self
            .language_buttons
            .iter_mut()
            .zip(language_metadatas)
            .enumerate()
            .map(|(i, (state, lang))| {
                let color = if Some(i) == selected_language_index {
                    super::selection_active_rgb()
                } else {
                    super::selection_inactive_rgb()
                };
                let button = Button::new(
                    state,
                    Row::with_children(vec![
                        Space::new(Length::FillPortion(5), Length::Units(0)).into(),
                        Text::new(lang.language_name.clone())
                            .width(Length::FillPortion(95))
                            .font(fonts.universal.id)
                            .size(fonts.universal.scale(25))
                            .vertical_alignment(iced::VerticalAlignment::Center)
                            .into(),
                    ]),
                )
                .style(style::button::Style::selection(
                    imgs.selection,
                    imgs.selection_hover,
                    imgs.selection_press,
                    Rgba::new(color.0, color.1, color.2, 192),
                ))
                .min_height(56)
                .on_press(Message::LanguageChanged(i));
                Row::with_children(vec![
                    Space::new(Length::FillPortion(3), Length::Units(0)).into(),
                    button.width(Length::FillPortion(92)).into(),
                    Space::new(Length::FillPortion(5), Length::Units(0)).into(),
                ])
            });

        for item in list_items {
            list = list.push(item);
        }

        let okay_button = Container::new(neat_button(
            &mut self.okay_button,
            i18n.get_msg("common-okay"),
            FILL_FRAC_TWO,
            button_style,
            Some(Message::OpenLanguageMenu),
        ))
        .center_x()
        .max_width(200);

        let content = Column::with_children(vec![title.into(), list.into(), okay_button.into()])
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::Center);

        let selection_menu = Container::new(content)
            .padding([8, 4])
            .max_width(350)
            .height(Length::Fill);

        selection_menu.into()
    }
}

#[derive(Default)]
pub struct LoginBanner {
    pub username: text_input::State,
    pub password: text_input::State,

    #[cfg(feature = "singleplayer")]
    singleplayer_button: button::State,
}

impl LoginBanner {
    fn view(
        &mut self,
        fonts: &Fonts,
        login_info: &LoginInfo,
        i18n: &Localization,
        button_style: style::button::Style,
    ) -> Element<'_, Message> {
        let input_text_size = fonts.universal.scale(INPUT_TEXT_SIZE);

        let banner_content = Column::with_children(vec![
            Column::with_children(vec![
                Container::new(
                    TextInput::new(
                        &mut self.username,
                        &i18n.get_msg("main-username"),
                        &login_info.username,
                        Message::Username,
                    )
                    .size(input_text_size)
                    .on_submit(Message::FocusPassword),
                )
                .width(Length::Units(INPUT_WIDTH))
                .padding([5, 7])
                .into(),
                Container::new(
                    TextInput::new(
                        &mut self.password,
                        &i18n.get_msg("main-password"),
                        &login_info.password,
                        Message::Password,
                    )
                    .size(input_text_size)
                    .password(),
                )
                .width(Length::Units(INPUT_WIDTH))
                .padding([5, 7])
                .into(),
            ])
            .spacing(5)
            .into(),
            Space::new(Length::Fill, Length::Units(8)).into(),
            Column::with_children(vec![
                #[cfg(feature = "singleplayer")]
                neat_button(
                    &mut self.singleplayer_button,
                    i18n.get_msg("common-singleplayer"),
                    FILL_FRAC_TWO,
                    button_style,
                    Some(Message::Singleplayer),
                ),
            ])
            .max_width(220)
            .height(Length::Units(64))
            .spacing(5)
            .into(),
        ])
        .width(Length::Fill)
        .align_items(Align::Center);

        Container::new(banner_content)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}
