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

mod common;

use common::*;

use plinth::mpd::playback::PlaybackClientError;
use plinth::ui::types::Message;

fn make_error_app(error: PlaybackClientError) -> plinth::ui::App {
    use plinth::config::AppSettings;
    use plinth::mpd::ReloadFlag;
    plinth::ui::App {
        state: plinth::ui::AppState::Error(error),
        settings: AppSettings::default(),
        reload_flag: ReloadFlag::new(),
    }
}

#[test]
fn app_error_state_dismiss_error_noop() {
    let mut app = make_error_app(PlaybackClientError::MPDConnectionError("test".into()));
    let task = app.update(Message::DismissError);
    let messages = extract_task_messages(task);
    assert!(messages.is_empty());
}

#[test]
fn app_error_state_ignores_playback_messages() {
    let mut app = make_error_app(PlaybackClientError::MPDConnectionError("test".into()));

    let task = app.update(Message::PlayPause);
    assert!(extract_task_messages(task).is_empty());

    let task = app.update(Message::GridAlbumClicked(0));
    assert!(extract_task_messages(task).is_empty());

    let task = app.update(Message::SetPlaybackState(
        mpd::State::Play,
        Some(0.0),
        Some(180.0),
        None,
    ));
    assert!(extract_task_messages(task).is_empty());
}

#[test]
fn app_error_state_main_window_returns_none() {
    let mut app = make_error_app(PlaybackClientError::MPDConnectionError("test".into()));
    assert!(app.main_window().is_none());
    assert!(app.playback_client_mut().is_none());
}