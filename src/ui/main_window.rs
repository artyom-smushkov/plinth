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

use super::album_grid::AlbumGrid;
use super::now_playing::NowPlayingWidget;
use super::player_control::PlayerControl;
use super::settings::SettingsWidget;
use crate::config::{AppSettings, ColorScheme};
use crate::mpd::types::{Album, AlbumSortConfig};
use crate::ui::types::Message;
use iced::Task;
use iced::{
    widget::{column, container},
    Element, Length,
};
use mpd::State;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    AlbumGrid,
    NowPlaying,
    Settings,
}

pub struct MainWindow {
    player_control: PlayerControl,
    album_grid: AlbumGrid,
    pub(crate) now_playing: NowPlayingWidget,
    settings_widget: SettingsWidget,
    playback_state: State,
    current_view: View,
    elapsed_secs: Option<f64>,
    duration_secs: Option<f64>,
    albums: Rc<Vec<Rc<Album>>>,
    albums_order: Rc<Vec<usize>>,
    settings: AppSettings,
}

impl MainWindow {
    pub fn new(
        albums: Rc<Vec<Rc<Album>>>,
        albums_order: Rc<Vec<usize>>,
        sort_config: AlbumSortConfig,
        settings: AppSettings,
    ) -> Self {
        let grid_thumbnail_size = settings.grid_thumbnail_size;
        let now_playing_thumbnail_size = settings.now_playing_thumbnail_size;

        Self {
            player_control: PlayerControl::new(),
            album_grid: AlbumGrid::new(albums.clone(), albums_order.clone(), &sort_config, grid_thumbnail_size),
            now_playing: NowPlayingWidget::new(albums.clone(), now_playing_thumbnail_size),
            settings_widget: SettingsWidget::new(settings.clone()),
            playback_state: State::Stop,
            current_view: View::AlbumGrid,
            elapsed_secs: None,
            duration_secs: None,
            albums,
            albums_order,
            settings,
        }
    }

    pub fn current_view(&self) -> View {
        self.current_view
    }

    pub fn playback_state(&self) -> State {
        self.playback_state
    }

    pub fn elapsed_secs(&self) -> Option<f64> {
        self.elapsed_secs
    }

    pub fn duration_secs(&self) -> Option<f64> {
        self.duration_secs
    }

    pub fn init(&mut self) -> iced::Task<Message> {
        self.album_grid.init()
    }

    pub fn view(&self, is_reload_pending: bool) -> Element<'_, Message> {
        let player_control = self.player_control.view(self.playback_state, self.elapsed_secs, self.duration_secs, &self.settings, self.current_view, is_reload_pending);

        let content = match self.current_view {
            View::AlbumGrid => self.album_grid.view(),
            View::NowPlaying => self.now_playing.view(),
            View::Settings => self.settings_widget.view(),
        };

        column![
            container(content).width(Length::Fill).height(Length::Fill),
            container(player_control)
            .width(Length::Fill)
            .height(Length::Fixed(64.0)),
        ]
            .spacing(0)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AlbumGridButtonClicked => {
                self.current_view = View::AlbumGrid;
                self.album_grid.rebuild(&self.settings.sort_config, self.settings.grid_thumbnail_size)
            }
            Message::NowPlayingButtonClicked => {
                self.current_view = View::NowPlaying;
                Task::none()
            }
            Message::SettingsButtonClicked => {
                self.current_view = View::Settings;
                Task::none()
            }
            Message::SetPlaybackState(state, elapsed, duration, _) => {
                self.playback_state = state;
                self.elapsed_secs = elapsed;
                self.duration_secs = duration;
                Task::none()
            }
            Message::ControlHovered(_) | Message::ControlUnhovered(_) => {
                self.player_control.update(message)
            }
            Message::GridAlbumClicked(_) | Message::GridThumbnailReady(_, _) => {
                self.album_grid.update(message)
            }
            Message::TrackClicked(_, _) | Message::TrackHovered(_, _) | Message::TrackUnhovered(_, _)
            | Message::SetCurrentTrack(_, _) | Message::NowPlayingThumbnailReady(_, _)
            | Message::SyncQueue(_) | Message::AlbumHovered(_) | Message::AlbumUnhovered(_)
            | Message::RemoveAlbumFromQueue(_) => {
                self.now_playing.update(message)
            }
            Message::SettingGridThumbnailSizeChanged(size) => {
                self.settings.grid_thumbnail_size = size;
                self.settings.save();
                self.settings_widget.set_settings(self.settings.clone());
                Task::none()
            }
            Message::SettingNowPlayingThumbnailSizeChanged(size) => {
                self.settings.now_playing_thumbnail_size = size;
                self.settings.save();
                self.now_playing.set_thumbnail_size(size);
                self.settings_widget.set_settings(self.settings.clone());
                Task::none()
            }
            Message::SettingColorschemeChanged(scheme) => {
                self.apply_colorscheme(scheme);
                self.settings.save();
                Task::none()
            }
            Message::SettingSortHighestChanged(field) => {
                self.settings.sort_config.highest = field;
                self.settings.save();
                self.settings_widget.set_settings(self.settings.clone());
                Task::none()
            }
            Message::SettingSortMiddleChanged(field) => {
                self.settings.sort_config.middle = field;
                self.settings.save();
                self.settings_widget.set_settings(self.settings.clone());
                Task::none()
            }
            Message::SettingSortLowestChanged(field) => {
                self.settings.sort_config.lowest = field;
                self.settings.save();
                self.settings_widget.set_settings(self.settings.clone());
                Task::none()
            }
            _ => Task::none()
        }
    }

    pub fn is_track_hovered(&self, album_idx: usize, track_idx: usize) -> bool {
        if let Some(playing_album) = self.now_playing.get_playing_album(album_idx)
            && let Some(track_widget) = playing_album.get_track_widget(track_idx)
        {
            return track_widget.is_hovered();
        }
        false
    }

    pub fn is_track_current(&self, album_idx: usize, track_idx: usize) -> bool {
        if let Some(playing_album) = self.now_playing.get_playing_album(album_idx)
            && let Some(track_widget) = playing_album.get_track_widget(track_idx)
        {
            return track_widget.is_current_track();
        }
        false
    }

    pub fn playing_album_count(&self) -> usize {
        self.now_playing.playing_album_count()
    }

    pub fn is_album_hovered(&self, album_idx: usize) -> bool {
        if let Some(playing_album) = self.now_playing.get_playing_album(album_idx) {
            return playing_album.is_hovered();
        }
        false
    }

    pub fn is_thumbnail_loaded(&self, album_idx: usize) -> bool {
        if let Some(playing_album) = self.now_playing.get_playing_album(album_idx) {
            return playing_album.album_widget.is_thumbnail_loaded();
        }
        false
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn apply_colorscheme(&mut self, scheme: ColorScheme) {
        self.settings.colorscheme = scheme;
        self.settings_widget.set_settings(self.settings.clone());
    }

    pub fn update_albums(
        &mut self,
        albums: Rc<Vec<Rc<Album>>>,
        albums_order: Rc<Vec<usize>>,
    ) -> Task<Message> {
        self.albums = albums.clone();
        self.albums_order = albums_order.clone();
        self.album_grid = AlbumGrid::new(
            albums,
            albums_order,
            &self.settings.sort_config,
            self.settings.grid_thumbnail_size,
        );
        self.now_playing = NowPlayingWidget::new(
            self.albums.clone(),
            self.settings.now_playing_thumbnail_size,
        );
        self.album_grid.init()
    }

    pub fn set_updating_db(&mut self, _updating_db: Option<u32>) {
    }
}