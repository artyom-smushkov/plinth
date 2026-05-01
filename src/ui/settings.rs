// This file is part of Plinth - album focused graphical mpd client.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// Copyright (C) 2026 Artem Smushkov <artem.smushkov@proton.me>

use crate::config::{AppSettings, ColorScheme};
use crate::mpd::types::AlbumSortField;
use crate::ui::types::Message;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::pick_list;
use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::Padding;

pub struct SettingsWidget {
    settings: AppSettings,
}

impl SettingsWidget {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn set_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = self.settings.theme();
        let extended = theme.extended_palette();
        let faded_color = extended.secondary.weak.color;
        let bg = extended.secondary.base.color;
        let text_color = extended.background.base.text;
        let dim_bg = extended.secondary.weak.color;
        let dim_text = extended.secondary.weak.color;

        let grid_current = self.settings.grid_thumbnail_size;
        let grid_minus = grid_current > 100;
        let grid_plus = grid_current < 1000;

        let np_current = self.settings.now_playing_thumbnail_size;
        let np_minus = np_current > 100;
        let np_plus = np_current < 2000;

        let grid_minus_btn = make_button(
            "-", grid_minus, bg, text_color, dim_bg, dim_text,
            Message::SettingGridThumbnailSizeChanged(grid_current.saturating_sub(20)),
        );
        let grid_plus_btn = make_button(
            "+", grid_plus, bg, text_color, dim_bg, dim_text,
            Message::SettingGridThumbnailSizeChanged(grid_current.saturating_add(20)),
        );

        let np_minus_btn = make_button(
            "-", np_minus, bg, text_color, dim_bg, dim_text,
            Message::SettingNowPlayingThumbnailSizeChanged(np_current.saturating_sub(20)),
        );
        let np_plus_btn = make_button(
            "+", np_plus, bg, text_color, dim_bg, dim_text,
            Message::SettingNowPlayingThumbnailSizeChanged(np_current.saturating_add(20)),
        );

        let grid_size_row = row![
            text("Grid Thumbnail Size").size(14).width(Length::Fill),
            grid_minus_btn,
            container(text(format!("{} px", grid_current)).size(14)).width(Length::Fixed(70.0)).center_x(Length::Fill),
            grid_plus_btn,
        ]
        .align_y(Alignment::Center)
        .spacing(8);

        let np_size_row = row![
            text("Now Playing Thumbnail Size").size(14).width(Length::Fill),
            np_minus_btn,
            container(text(format!("{} px", np_current)).size(14)).width(Length::Fixed(70.0)).center_x(Length::Fill),
            np_plus_btn,
        ]
        .align_y(Alignment::Center)
        .spacing(8);

        let cs_options = ColorScheme::all_variants().to_vec();
        let cs_selected = Some(self.settings.colorscheme);
        let colorscheme_picker = row![
            text("Colorscheme").size(14).width(Length::Fill),
            container(
                pick_list(cs_options, cs_selected, Message::SettingColorschemeChanged)
                    .text_size(14)
            )
            .padding(4),
        ]
        .align_y(Alignment::Center);

        let sort_options = SortOption::all_with_none();
        let sort_highest_sel = sort_options.iter().find(|o| o.field() == self.settings.sort_config.highest).cloned();
        let sort_highest_picker = row![
            text("Sort - Highest").size(14).width(Length::Fill),
            container(
                pick_list(sort_options.clone(), sort_highest_sel, |opt: SortOption| {
                    Message::SettingSortHighestChanged(opt.field())
                })
                .text_size(14)
            )
            .padding(4),
        ]
        .align_y(Alignment::Center);

        let sort_middle_sel = sort_options.iter().find(|o| o.field() == self.settings.sort_config.middle).cloned();
        let sort_middle_picker = row![
            text("Sort - Middle").size(14).width(Length::Fill),
            container(
                pick_list(sort_options.clone(), sort_middle_sel, |opt: SortOption| {
                    Message::SettingSortMiddleChanged(opt.field())
                })
                .text_size(14)
            )
            .padding(4),
        ]
        .align_y(Alignment::Center);

        let sort_lowest_options = AlbumSortField::all_variants().to_vec();
        let sort_lowest_picker = row![
            text("Sort - Lowest").size(14).width(Length::Fill),
            container(
                pick_list(sort_lowest_options, Some(self.settings.sort_config.lowest), Message::SettingSortLowestChanged)
                    .text_size(14)
            )
            .padding(4),
        ]
        .align_y(Alignment::Center);

        let content = column![
            container(text("Settings").size(24)).padding(Padding { bottom: 20.0, ..Padding::ZERO }),
            container(text("Appearance").size(18).color(faded_color))
                .padding(Padding { bottom: 8.0, ..Padding::ZERO }),
            grid_size_row,
            np_size_row,
            colorscheme_picker,
            container(text("Album Grid").size(18).color(faded_color))
                .padding(Padding { top: 20.0, bottom: 8.0, ..Padding::ZERO }),
            sort_highest_picker,
            sort_middle_picker,
            sort_lowest_picker,
        ]
        .padding(20)
        .spacing(12);

        scrollable(content).into()
    }
}

fn make_button(
    label: &'static str,
    enabled: bool,
    bg: iced::Color,
    text_color: iced::Color,
    dim_bg: iced::Color,
    dim_text: iced::Color,
    message: Message,
) -> Element<'static, Message> {
    let btn_content = if enabled {
        container(text(label).size(18))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(bg.into()),
                text_color: Some(text_color),
                ..Default::default()
            })
            .into()
    } else {
        container(text(label).size(18).color(dim_text))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(dim_bg.into()),
                ..Default::default()
            })
            .into()
    };

    if enabled {
        iced::widget::mouse_area(btn_content)
            .on_press(message)
            .into()
    } else {
        btn_content
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SortOption {
    None,
    Field(AlbumSortField),
}

impl SortOption {
    fn all_with_none() -> Vec<Self> {
        let mut vec = vec![SortOption::None];
        for field in AlbumSortField::all_variants() {
            vec.push(SortOption::Field(*field));
        }
        vec
    }

    fn field(&self) -> Option<AlbumSortField> {
        match self {
            SortOption::None => None,
            SortOption::Field(field) => Some(*field),
        }
    }
}

impl std::fmt::Display for SortOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOption::None => write!(f, "None"),
            SortOption::Field(field) => write!(f, "{}", field.label()),
        }
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl AlbumSortField {
    fn all_variants() -> &'static [AlbumSortField] {
        &[
            AlbumSortField::Genre,
            AlbumSortField::Artist,
            AlbumSortField::Date,
            AlbumSortField::Name,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            AlbumSortField::Name => "Name",
            AlbumSortField::Artist => "Artist",
            AlbumSortField::Date => "Date",
            AlbumSortField::Genre => "Genre",
        }
    }
}

impl std::fmt::Display for AlbumSortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}