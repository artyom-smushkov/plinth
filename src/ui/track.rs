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

use crate::{config::AppSettings, mpd::types::Track, ui::types::Message};
use iced::{
    Alignment, Background, Color, Element, Length, Task, widget::{row, text}
};

pub struct TrackWidget {
    track: Track,
    queue_index: usize,
    pub (crate) track_index: usize,
    is_current_track: bool,
    hovered: bool,
}

impl TrackWidget {
    pub fn new(track: Track, queue_index: usize, track_index: usize, is_current: bool) -> Self {
        Self {
            track,
            queue_index,
            track_index,
            is_current_track: is_current,
            hovered: false,
        }
    }

    pub fn format_duration(&self) -> String {
        self.track
            .mpd_song
            .duration
            .map(|l| {
                let total_seconds = l.as_secs();
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;
                format!("{:02}:{:02}", minutes, seconds)
            })
            .unwrap_or_else(|| "--:--".to_string())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let track_number = format!("{}", self.track.track_number.unwrap_or(0));
        let duration = self.format_duration();

        let settings = AppSettings::default();
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let highlight_color = extended.background.weak.color;
        let faded_color = extended.secondary.weak.color;
        let is_highlighted = self.is_current_track || self.hovered;

        iced::widget::mouse_area(
            iced::widget::container(
                row![
                    text(track_number.clone()).color(faded_color),
                    text(&self.track.title).width(Length::FillPortion(3)),
                    text(duration).width(Length::Fixed(60.0)).color(faded_color),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .padding([4, 8]),
            )
            .width(Length::Fill)
            .style(move |_| {
                let background = if is_highlighted {
                    Background::Color(highlight_color)
                } else {
                    Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))
                };
                iced::widget::container::Style::default().background(background)
            }),
        )
        .on_press(Message::TrackClicked(self.queue_index, self.track_index))
        .on_enter(Message::TrackHovered(self.queue_index, self.track_index))
        .on_exit(Message::TrackUnhovered(self.queue_index, self.track_index))
        .into()
    }

    pub fn is_current_track(&self) -> bool {
        self.is_current_track
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrackHovered(_album_idx, track_idx) => {
                if self.track_index == track_idx {
                    self.hovered = true;
                }
            }
            Message::TrackUnhovered(_album_idx, track_idx) => {
                if self.track_index == track_idx {
                    self.hovered = false;
                }
            }
            Message::TrackStartedPlaying(_album_idx, track_idx) => {
                if self.track_index == track_idx {
                    self.is_current_track = true;
                }
            }
            Message::TrackEndedPlaying(_album_idx, track_idx) => {
                if self.track_index == track_idx {
                    self.is_current_track = false;
                }
            }
            _ => {}
        };
        Task::none()
    }
}
