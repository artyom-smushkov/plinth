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

use std::cell::RefCell;

use crate::config::ColorScheme;
use crate::mpd::playback::PlaybackClientError;
use crate::mpd::types::AlbumSortField;
use crate::ui::main_window::View;
use iced::widget::operation::AbsoluteOffset;
use mpd::State;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    AlbumGrid,
    NowPlaying,
    Settings,
    PreviousSong,
    PlayPause,
    NextSong,
    ClearQueue,
    ReloadDatabase,
}

#[derive(Debug, Clone)]
pub enum Message {
    DismissError,
    Exit,
    PlaybackStateUpdate {
        state: State,
        current_song_position: Option<usize>,
        elapsed_secs: Option<f64>,
        duration_secs: Option<f64>,
    },
    SetPlaybackState(State, Option<f64>, Option<f64>, Option<usize>),
    AlbumGridButtonClicked,
    NowPlayingButtonClicked,
    SettingsButtonClicked,
    ViewScrolled(View, AbsoluteOffset),
    PreviousSong,
    PlayPause,
    NextSong,
    ControlHovered(Control),
    ControlUnhovered(Control),
    GridAlbumClicked(usize),
    GridThumbnailReady(String, Result<(String, Vec<u8>), (String, PlaybackClientError)>),
    SyncQueue(RefCell<Vec<usize>>),
    RemoveAlbumFromQueue(usize),
    ClearQueue,
    ReloadDatabase,
    UpdatingDatabase(Option<u32>),
    DatabaseUpdated,
    SetCurrentTrack(usize, usize),
    NowPlayingThumbnailReady(String, Result<(String, Vec<u8>), (String, PlaybackClientError)>),
    TrackClicked(usize, usize),
    TrackHovered(usize, usize),
    TrackUnhovered(usize, usize),
    AlbumHovered(usize),
    AlbumUnhovered(usize),
    TrackStartedPlaying(usize, usize),
    TrackEndedPlaying(usize, usize),
    SeekTo(f64),
    ProgressSeek(f32),
    SettingGridThumbnailSizeChanged(u32),
    SettingNowPlayingThumbnailSizeChanged(u32),
    SettingColorschemeChanged(ColorScheme),
    SettingSortHighestChanged(Option<AlbumSortField>),
    SettingSortMiddleChanged(Option<AlbumSortField>),
    SettingSortLowestChanged(AlbumSortField),
}