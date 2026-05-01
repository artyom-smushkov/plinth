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

use std::rc::Rc;

use crate::mpd::types::Album;
use iced::{
    widget::{column, container, text},
    Alignment, Element, Length,
};

pub struct AlbumInfoWidget {
    album: Rc<Album>,
}

impl AlbumInfoWidget {
    pub fn new(album: Rc<Album>) -> Self {
        Self { album }
    }

    pub fn view(&self) -> Element<'_, ()> {
        let settings = crate::config::AppSettings::default();
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let faded_color = extended.secondary.weak.color;

        let total_duration = self.calculate_total_duration();

        let info_column = column![
            text(self.album.artist.clone()).color(faded_color),
            text(self.album.name.clone()).color(faded_color),
            text(format!(
                "{} | {}",
                self.album.genre,
                self.album.date.as_deref().unwrap_or("Unknown")
            ))
            .color(faded_color),
            text(format!(
                "{} tracks | {}",
                self.album.tracks.len(),
                total_duration
            ))
            .color(faded_color),
        ]
        .spacing(4)
        .width(Length::Fixed(300.0))
        .align_x(Alignment::Start);

        container(info_column)
            .padding(16)
            .width(Length::Fixed(300.0))
            .into()
    }

    fn calculate_total_duration(&self) -> String {
        let seconds: u64 = self
            .album
            .tracks
            .iter()
            .filter_map(|t| t.mpd_song.duration)
            .map(|d| d.as_secs())
            .sum();

        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    }
}