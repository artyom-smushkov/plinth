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

use super::album::{AlbumDisplayOption, AlbumWidget};
use super::album_info::AlbumInfoWidget;
use super::player_control::{ControlIcon, Icon};
use super::track::TrackWidget;
use crate::config::AppSettings;
use crate::mpd::types::Album;
use crate::ui::types::Message;
use iced::{
    Background, Color, Element, Length, Task,
    widget::{canvas, column, container, mouse_area, row, scrollable, Column},
};

pub struct PlayingAlbumWidget {
    pub(crate) album_widget: AlbumWidget,
    pub(crate) queue_index: usize,
    info_widget: AlbumInfoWidget,
    pub(crate) track_widgets: Vec<TrackWidget>,
    current_track_index: usize,
    #[allow(dead_code)]
    hovered_index: Option<usize>,
    hovered: bool,
}

impl PlayingAlbumWidget {
      pub fn new(queue_index: usize, album: Rc<Album>, now_playing_thumbnail_size: u32) -> Self {
        let album_widget = AlbumWidget::new(album.clone(), None, AlbumDisplayOption::NowPlaying, now_playing_thumbnail_size);
        let info_widget = AlbumInfoWidget::new(album.clone());
        let track_widgets = album
            .tracks
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, track)| TrackWidget::new(track, queue_index, index, false))
            .collect();
        Self {
            album_widget,
            queue_index,
            info_widget,
            track_widgets,
            current_track_index: 0,
            hovered_index: None,
            hovered: false,
        }
    }

    pub fn current_track_index(&self) -> usize {
        self.current_track_index
    }

    pub fn get_track_widget(&self, index: usize) -> Option<&TrackWidget> {
        self.track_widgets.get(index)
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn view(&self) -> Element<'_, Message> {
        let album_widget = self.album_widget.view();
        let info_widget = self.info_widget.view().map(|_| Message::TrackClicked(0, 0));
        let track_widgets = self
            .track_widgets
            .iter()
            .map(|widget| widget.view());
        let tracklist = scrollable(Column::with_children(track_widgets).spacing(8).padding(10));

        let settings = AppSettings::default();
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let hover_bg = extended.background.weak.color;
        let normal_text = extended.secondary.weak.color;
        let hover_text = extended.background.base.text;
        let qi = self.queue_index;
        let is_hovered = self.hovered;

        let remove_button = mouse_area(
            container(
                canvas(ControlIcon {
                    icon: Icon::Close,
                    color: if is_hovered { hover_text } else { normal_text },
                })
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
            )
            .padding([2, 8])
            .style(move |_| {
                let bg = if is_hovered {
                    Background::Color(hover_bg)
                } else {
                    Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))
                };
                container::Style::default().background(bg)
            }),
        )
        .on_press(Message::RemoveAlbumFromQueue(qi));

        let upper_row = row![
            container(info_widget).width(Length::Fill),
            remove_button
        ];
        let right_column = column![upper_row, tracklist];

        let content_row = row![album_widget, right_column]
            .spacing(20);

        mouse_area(container(content_row).width(Length::Fill))
            .on_enter(Message::AlbumHovered(qi))
            .on_exit(Message::AlbumUnhovered(qi))
            .into()
    }

    pub fn init(&mut self) -> Task<Message> {
        self.album_widget.init()
    }

    pub fn set_thumbnail_size(&mut self, size: u32) -> Task<Message> {
        self.album_widget.set_thumbnail_size(size)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrackClicked(queue_index, track_index) => {
                Task::done(Message::TrackClicked(queue_index, track_index))
            }
            Message::TrackHovered(album_idx, track_idx) => {
                if let Some(track_widget) = self.track_widgets.get_mut(track_idx) {
                    return track_widget.update(Message::TrackHovered(album_idx, track_idx));
                }
                Task::none()
            }
            Message::TrackUnhovered(album_idx, track_idx) => {
                if let Some(track_widget) = self.track_widgets.get_mut(track_idx) {
                    return track_widget.update(Message::TrackUnhovered(album_idx, track_idx));
                }
                Task::none()
            }
            Message::SetCurrentTrack(album_idx, track_idx) => {
                if self.queue_index == album_idx {
                    self.current_track_index = track_idx;
                    for (idx, widget) in self.track_widgets.iter_mut().enumerate() {
                        let msg = if idx == track_idx {
                            Message::TrackStartedPlaying(album_idx, idx)
                        } else {
                            Message::TrackEndedPlaying(album_idx, idx)
                        };
                        let _ = widget.update(msg);
                    }
                }
                Task::none()
            }
            Message::TrackStartedPlaying(album_idx, track_idx) => {
                if let Some(track_widget) = self.track_widgets.get_mut(track_idx) {
                    return track_widget.update(Message::TrackStartedPlaying(album_idx, track_idx));
                }
                Task::none()
            }
            Message::TrackEndedPlaying(album_idx, track_idx) => {
                if let Some(track_widget) = self.track_widgets.get_mut(track_idx) {
                    return track_widget.update(Message::TrackEndedPlaying(album_idx, track_idx));
                }
                Task::none()
            }
            Message::NowPlayingThumbnailReady(widget_id, result) => {
                self.album_widget.update(Message::NowPlayingThumbnailReady(widget_id, result))
            }
            Message::AlbumHovered(album_idx) => {
                if self.queue_index == album_idx {
                    self.hovered = true;
                }
                Task::none()
            }
            Message::AlbumUnhovered(album_idx) => {
                if self.queue_index == album_idx {
                    self.hovered = false;
                }
                Task::none()
            }
            Message::RemoveAlbumFromQueue(queue_index) => {
                if self.queue_index == queue_index {
                    return Task::done(Message::RemoveAlbumFromQueue(queue_index));
                }
                Task::none()
            }
            _ => Task::none()
        }
    }
}
