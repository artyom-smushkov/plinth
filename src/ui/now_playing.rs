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

use std::{cell::RefCell, rc::Rc};

use super::playing_album::PlayingAlbumWidget;
use crate::mpd::types::Album;
use crate::ui::types::Message;
use iced::{
    widget::{scrollable, Column},
    Element, Task,
};

pub struct NowPlayingWidget {
    albums: Rc<Vec<Rc<Album>>>,
    playing_album_widgets: Vec<PlayingAlbumWidget>,
    now_playing_thumbnail_size: u32,
}

impl NowPlayingWidget {
    pub fn new(albums: Rc<Vec<Rc<Album>>>, now_playing_thumbnail_size: u32) -> Self {
        Self {
            albums,
            playing_album_widgets: Vec::new(),
            now_playing_thumbnail_size,
        }
    }

    pub fn set_thumbnail_size(&mut self, size: u32) -> Task<Message> {
        if size == self.now_playing_thumbnail_size {
            return Task::none();
        }
        self.now_playing_thumbnail_size = size;
        Task::batch(
            self.playing_album_widgets
                .iter_mut()
                .map(|widget| widget.set_thumbnail_size(size))
        )
    }

    pub fn playing_album_count(&self) -> usize {
        self.playing_album_widgets.len()
    }

    pub fn get_playing_album(&self, index: usize) -> Option<&PlayingAlbumWidget> {
        self.playing_album_widgets.get(index)
    }

    pub fn set_queue(&mut self, queue: RefCell<Vec<usize>>) {
        let np_size = self.now_playing_thumbnail_size;
        self.playing_album_widgets = queue
            .borrow()
            .iter()
            .enumerate()
            .map(|(queue_index, i)| PlayingAlbumWidget::new(queue_index, self.albums[*i].clone(), np_size))
            .collect();
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::batch(
            self.playing_album_widgets
                .iter_mut()
                .map(|widget| widget.init())
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let playing_widgets = self
            .playing_album_widgets
            .iter()
            .map(|widget| widget.view());
        scrollable(
            Column::with_children(playing_widgets)
                .spacing(20)
                .padding(20),
        )
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrackClicked(queue_index, track_index) => {
                Task::done(Message::TrackClicked(queue_index, track_index))
            }
            Message::TrackHovered(album_idx, track_idx) => {
                if let Some(playing_album) = self.playing_album_widgets.get_mut(album_idx) {
                    return playing_album.update(Message::TrackHovered(album_idx, track_idx));
                }
                Task::none()
            }
            Message::TrackUnhovered(album_idx, track_idx) => {
                if let Some(playing_album) = self.playing_album_widgets.get_mut(album_idx) {
                    return playing_album.update(Message::TrackUnhovered(album_idx, track_idx));
                }
                Task::none()
            }
            Message::SetCurrentTrack(album_idx, track_idx) => {
                if let Some(playing_album) = self.playing_album_widgets.get_mut(album_idx) {
                    return playing_album.update(Message::SetCurrentTrack(album_idx, track_idx));
                }
                Task::none()
            }
            Message::NowPlayingThumbnailReady(widget_id, result) => {
                for playing_album in self.playing_album_widgets.iter_mut() {
                    if playing_album.album_widget.widget_id == widget_id {
                        return playing_album.update(Message::NowPlayingThumbnailReady(widget_id, result));
                    }
                }
                Task::none()
            }
            Message::SyncQueue(queue) => {
                self.set_queue(queue);
                self.init()
            }
            Message::AlbumHovered(album_idx) => {
                if let Some(playing_album) = self.playing_album_widgets.get_mut(album_idx) {
                    return playing_album.update(Message::AlbumHovered(album_idx));
                }
                Task::none()
            }
            Message::AlbumUnhovered(album_idx) => {
                if let Some(playing_album) = self.playing_album_widgets.get_mut(album_idx) {
                    return playing_album.update(Message::AlbumUnhovered(album_idx));
                }
                Task::none()
            }
            Message::RemoveAlbumFromQueue(queue_index) => {
                if let Some(playing_album) = self.playing_album_widgets.get(queue_index)
                    && playing_album.queue_index == queue_index
                {
                    return Task::done(Message::RemoveAlbumFromQueue(queue_index));
                }
                Task::none()
            }
            _ => Task::none()
        }
    }
}