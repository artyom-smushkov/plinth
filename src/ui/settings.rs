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
use crate::ui::main_window::View;
use crate::ui::types::Message;
use iced::border::radius;
use iced::widget::column;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::slider;
use iced::widget::text;
use iced::widget::Id;
use iced::{Alignment, Element, Length};

pub const GRID_THUMBNAIL_MIN: u32 = 100;
pub const GRID_THUMBNAIL_MAX: u32 = 1000;
pub const NOW_PLAYING_THUMBNAIL_MIN: u32 = 100;
pub const NOW_PLAYING_THUMBNAIL_MAX: u32 = 2000;
pub const THUMBNAIL_STEP: u32 = 20;

const CONTENT_MAX_WIDTH: f32 = 640.0;
const CONTENT_PADDING: f32 = 24.0;
const CONTENT_SPACING: f32 = 16.0;
const LABEL_WIDTH: f32 = 230.0;
const VALUE_WIDTH: f32 = 70.0;
const CARD_PADDING: f32 = 20.0;
const CARD_RADIUS: f32 = 12.0;
const ROW_SPACING: f32 = 14.0;

const BOLD_FONT: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..crate::ui::UI_FONT
};

pub struct SettingsWidget {
    settings: AppSettings,
}

impl SettingsWidget {
    pub const SCROLLABLE_ID: Id = Id::new("settings");

    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn set_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    pub fn clamped_grid_thumbnail_size(&self) -> u32 {
        self.settings
            .grid_thumbnail_size
            .clamp(GRID_THUMBNAIL_MIN, GRID_THUMBNAIL_MAX)
    }

    pub fn clamped_now_playing_thumbnail_size(&self) -> u32 {
        self.settings
            .now_playing_thumbnail_size
            .clamp(NOW_PLAYING_THUMBNAIL_MIN, NOW_PLAYING_THUMBNAIL_MAX)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = self.settings.theme();
        let extended = theme.extended_palette();
        let text_color = extended.background.base.text;
        let faded_color = mix(text_color, extended.background.base.color, 0.35);
        let card_bg = extended.background.weaker.color;

        let grid_size = self.clamped_grid_thumbnail_size();
        let np_size = self.clamped_now_playing_thumbnail_size();

        let grid_slider = slider(
            GRID_THUMBNAIL_MIN as f32..=GRID_THUMBNAIL_MAX as f32,
            grid_size as f32,
            |value: f32| Message::SettingGridThumbnailSizeChanged(value as u32),
        )
        .step(THUMBNAIL_STEP as f32);

        let np_slider = slider(
            NOW_PLAYING_THUMBNAIL_MIN as f32..=NOW_PLAYING_THUMBNAIL_MAX as f32,
            np_size as f32,
            |value: f32| Message::SettingNowPlayingThumbnailSizeChanged(value as u32),
        )
        .step(THUMBNAIL_STEP as f32);

        let cs_options = ColorScheme::all_variants().to_vec();
        let cs_selected = Some(self.settings.colorscheme);
        let cs_picker = pick_list(cs_options, cs_selected, Message::SettingColorschemeChanged)
            .text_size(14);

        let sort_options = SortOption::all_with_none();
        let sort_highest_sel = sort_options
            .iter()
            .find(|o| o.field() == self.settings.sort_config.highest)
            .cloned();
        let sort_highest_picker = pick_list(sort_options.clone(), sort_highest_sel, |opt: SortOption| {
            Message::SettingSortHighestChanged(opt.field())
        })
        .text_size(14);

        let sort_middle_sel = sort_options
            .iter()
            .find(|o| o.field() == self.settings.sort_config.middle)
            .cloned();
        let sort_middle_picker = pick_list(sort_options.clone(), sort_middle_sel, |opt: SortOption| {
            Message::SettingSortMiddleChanged(opt.field())
        })
        .text_size(14);

        let sort_lowest_options = AlbumSortField::all_variants().to_vec();
        let sort_lowest_picker = pick_list(
            sort_lowest_options,
            Some(self.settings.sort_config.lowest),
            Message::SettingSortLowestChanged,
        )
        .text_size(14);

        let appearance = section_card(
            "Appearance",
            column![
                slider_row(
                    "Grid Thumbnail Size",
                    grid_slider.into(),
                    grid_size,
                    text_color,
                    text_color,
                ),
                slider_row(
                    "Now Playing Thumbnail Size",
                    np_slider.into(),
                    np_size,
                    text_color,
                    text_color,
                ),
                setting_row("Colorscheme", cs_picker.into(), text_color),
            ]
            .spacing(ROW_SPACING)
            .into(),
            faded_color,
            card_bg,
        );

        let album_grid = section_card(
            "Album Grid",
            column![
                setting_row("Sort - Highest", sort_highest_picker.into(), text_color),
                setting_row("Sort - Middle", sort_middle_picker.into(), text_color),
                setting_row("Sort - Lowest", sort_lowest_picker.into(), text_color),
            ]
            .spacing(ROW_SPACING)
            .into(),
            faded_color,
            card_bg,
        );

        let content = column![
            text("Settings").size(24).font(BOLD_FONT),
            appearance,
            album_grid,
        ]
        .padding(CONTENT_PADDING)
        .spacing(CONTENT_SPACING)
        .max_width(CONTENT_MAX_WIDTH);

        scrollable(
            container(content)
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Start),
        )
        .id(Self::SCROLLABLE_ID)
        .on_scroll(|viewport| {
            Message::ViewScrolled(View::Settings, viewport.absolute_offset())
        })
        .into()
    }
}

fn mix(a: iced::Color, b: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: a.r + (b.r - a.r) * factor,
        g: a.g + (b.g - a.g) * factor,
        b: a.b + (b.b - a.b) * factor,
        a: 1.0,
    }
}

fn section_card<'a>(
    title: &'a str,
    content: Element<'a, Message>,
    faded_color: iced::Color,
    card_bg: iced::Color,
) -> Element<'a, Message> {
    let header = text(title).size(14).font(BOLD_FONT).color(faded_color);
    container(column![header, content].spacing(ROW_SPACING).padding(CARD_PADDING).width(Length::Fill))
        .width(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(card_bg.into()),
            border: iced::border::Border {
                radius: radius(CARD_RADIUS),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn setting_row<'a>(label: &'a str, control: Element<'a, Message>, text_color: iced::Color) -> Element<'a, Message> {
    row![
        text(label).size(14).color(text_color).width(Length::Fixed(LABEL_WIDTH)),
        control,
    ]
    .align_y(Alignment::Center)
    .spacing(ROW_SPACING)
    .into()
}

fn slider_row<'a>(
    label: &'a str,
    slider: Element<'a, Message>,
    value: u32,
    text_color: iced::Color,
    dim_color: iced::Color,
) -> Element<'a, Message> {
    row![
        text(label).size(14).color(text_color).width(Length::Fixed(LABEL_WIDTH)),
        row![
            slider,
            container(text(format!("{} px", value)).size(14).color(dim_color))
                .width(Length::Fixed(VALUE_WIDTH))
                .align_x(Alignment::End),
        ]
        .align_y(Alignment::Center)
        .spacing(ROW_SPACING)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(ROW_SPACING)
    .into()
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
