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

use crate::mpd::playback::PlaybackClientError;
use crate::mpd::types::Album;
use crate::ui::types::Message;
use iced::{task::Task, widget::image::Handle};
use iced::{widget::image, Element, Length};
use mpd::Song;


async fn generate_thumbnail(
    widget_id: String, first_song: Song, album_artist: String, album_name: String, size: u32
) -> Result<(String, Vec<u8>), (String, PlaybackClientError)> {
    let cover_data_result = crate::mpd::library::get_thumbnail(
        &first_song,
        &album_artist,
        &album_name,
        size,
    );

    match cover_data_result {
        Ok(data) => Ok((widget_id, data)),
        Err(e) => Err((widget_id, e)),
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AlbumDisplayOption {
    Grid,
    NowPlaying,
}

pub enum ThumbnailState {
    NotLoaded,
    Loading,
    Loaded { handle: Handle },
    Error,
}

pub struct AlbumWidget {
    pub widget_id: String,
    album: Rc<Album>,
    index: Option<usize>,
    display_option: AlbumDisplayOption,
    thumbnail_state: ThumbnailState,
    thumbnail_size: u32,
}

impl AlbumWidget {
    pub fn new(
        album: Rc<Album>,
        index: Option<usize>,
        display_option: AlbumDisplayOption,
        thumbnail_size: u32,
    ) -> Self {
        let widget_id = match index {
            Some(i) => format!("grid-{i}"),
            None => format!("now-playing-{}-{}", album.artist, album.name),
        };

        Self {
            widget_id,
            album,
            index,
            display_option,
            thumbnail_state: ThumbnailState::NotLoaded,
            thumbnail_size,
        }
    }

    pub fn init(&mut self) -> Task<Message> {
        let widget_id = self.widget_id.clone();
        let display_option = self.display_option;
        let task = Task::perform(
            generate_thumbnail(
                widget_id.clone(),
                self.album.tracks[0].mpd_song.clone(),
                self.album.artist.clone(),
                self.album.name.clone(),
                self.thumbnail_size
            ),
            move |result| {
                match display_option {
                    AlbumDisplayOption::Grid => Message::GridThumbnailReady(widget_id, result),
                    AlbumDisplayOption::NowPlaying => Message::NowPlayingThumbnailReady(widget_id, result),
                }
            },
        );
        self.thumbnail_state = ThumbnailState::Loading;
        task
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cover_size = self.thumbnail_size as f32;

        let content: Element<'_, Message> = match &self.thumbnail_state {
            ThumbnailState::NotLoaded | ThumbnailState::Loading => {
                let placeholder_color = iced::Color::from_rgb8(75, 75, 80);
                iced::widget::center(
                    iced::widget::container(iced::widget::text("Loading...").size(24))
                        .width(Length::Fixed(cover_size))
                        .height(Length::Fixed(cover_size))
                        .style(move |_theme| iced::widget::container::Style {
                            background: Some(placeholder_color.into()),
                            ..Default::default()
                        }),
                )
                    .into()
            }
            ThumbnailState::Loaded { handle } => {
                let image_widget = image(handle)
                    .width(Length::Fixed(cover_size))
                    .height(Length::Fixed(cover_size));

                if let Some(index) = self.index {
                    iced::widget::mouse_area(image_widget)
                        .on_press(Message::GridAlbumClicked(index))
                        .into()
                } else {
                    image_widget.into()
                }
            }
            ThumbnailState::Error => {
                let placeholder_color = iced::Color::from_rgb8(150, 50, 50);
                iced::widget::center(
                    iced::widget::container(iced::widget::text("Error").size(24))
                        .width(Length::Fixed(cover_size))
                        .height(Length::Fixed(cover_size))
                        .style(move |_theme| iced::widget::container::Style {
                            background: Some(placeholder_color.into()),
                            ..Default::default()
                        }),
                )
                    .into()
            }
        };

        if let Some(index) = self.index {
            match &self.thumbnail_state {
                ThumbnailState::NotLoaded | ThumbnailState::Loading | ThumbnailState::Error => {
                    iced::widget::mouse_area(content)
                        .on_press(Message::GridAlbumClicked(index))
                        .into()
                }
                _ => content,
            }
        } else {
            content
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let widget_id = self.widget_id.clone();
        match message {
            Message::GridThumbnailReady(id, result) if id == widget_id => {
                self.handle_thumbnail(result);
            }
            Message::NowPlayingThumbnailReady(id, result) if id == widget_id => {
                self.handle_thumbnail(result);
            }
            _ => {}
        }
        Task::none()
    }

    pub fn is_thumbnail_loaded(&self) -> bool {
        matches!(self.thumbnail_state, ThumbnailState::Loaded { .. })
    }

    fn handle_thumbnail(&mut self, result: Result<(String, Vec<u8>), (String, PlaybackClientError)>) {
        match result {
            Ok(res) => {
                self.thumbnail_state = ThumbnailState::Loaded{ handle: Handle::from_bytes(res.1)};
            }
            Err(_) => {
                self.thumbnail_state = ThumbnailState::Error;
            }
        }
    }
}