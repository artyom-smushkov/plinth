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

use itertools::Itertools;
use mpd::Song;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlbumSortField {
    Name,
    Artist,
    Date,
    Genre,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlbumSortConfig {
    pub highest: Option<AlbumSortField>,
    pub middle: Option<AlbumSortField>,
    pub lowest: AlbumSortField,
}

impl Default for AlbumSortConfig {
    fn default() -> Self {
        Self {
            highest: Some(AlbumSortField::Genre),
            middle: Some(AlbumSortField::Artist),
            lowest: AlbumSortField::Date,
        }
    }
}

impl AlbumSortConfig {
    pub fn levels(&self) -> impl Iterator<Item = AlbumSortField> + '_ {
        self.highest.into_iter().chain(self.middle).chain(Some(self.lowest))
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub cd_number: Option<u32>,
    pub track_number: Option<u32>,
    pub mpd_song: Song,
}

pub struct Album {
    pub name: String,
    pub artist: String,
    pub genre: String,
    pub date: Option<String>,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn sort_key(&self, field: AlbumSortField) -> String {
        match field {
            AlbumSortField::Name => self.name.to_lowercase(),
            AlbumSortField::Artist => self.artist.to_lowercase(),
            AlbumSortField::Genre => self.genre.to_lowercase().split(' ').rev().join(" "),
            AlbumSortField::Date => self.date.clone().unwrap_or_default().to_lowercase(),
        }
    }

    pub fn group_label(&self, field: AlbumSortField) -> &str {
        match field {
            AlbumSortField::Name => &self.name,
            AlbumSortField::Artist => &self.artist,
            AlbumSortField::Genre => &self.genre,
            AlbumSortField::Date => self.date.as_deref().unwrap_or("Unknown"),
        }
    }
}

#[derive(Clone)]
pub struct PlaybackCursor {
    pub queued_album_index: usize,
    pub track_index: usize,
    pub position_ms: u32,
}