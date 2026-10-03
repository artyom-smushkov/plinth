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

use std::cell::RefCell;

use mpd::status::State;
use plinth::ui::types::Message;

#[test]
fn app_play_pause_toggles_client_state() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_state(State::Play);
    let mut app = make_app(vec![album], backend);

    let task = app.update(Message::PlayPause);
    for msg in extract_task_messages(task) {
        let _ = app.update(msg);
    }
    let mw = app.main_window().unwrap();
    assert_eq!(mw.playback_state(), State::Pause);
}

#[test]
fn app_play_pause_from_stop_no_change() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::PlayPause);
    let mw = app.main_window().unwrap();
    assert_eq!(mw.playback_state(), State::Stop);
}

#[test]
fn app_seek_to_delegates_to_client() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(0.0, 180.0);
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::SeekTo(60.0));
    let client = app.playback_client_mut().unwrap();
    let (elapsed, duration) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(60.0));
    assert_eq!(duration, Some(180.0));
}

#[test]
fn app_track_click_updates_client_cursor() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
        ],
    );
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.playback_client_mut().unwrap().play_album(0);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let task = app.update(Message::TrackClicked(0, 1));
    for msg in extract_task_messages(task) {
        let _ = app.update(msg);
    }
    let mw = app.main_window().unwrap();
    assert!(mw.is_track_current(0, 1));
}

#[test]
fn app_grid_album_click_adds_to_queue() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1.clone(), album2.clone()], backend);

    let _ = app.playback_client_mut().unwrap().play_album(0);

    let task = app.update(Message::GridAlbumClicked(1));
    for msg in extract_task_messages(task) {
        let _ = app.update(msg);
    }
    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 2);
}

#[test]
fn app_progress_seek_with_known_duration() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(45.0, 180.0);
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ProgressSeek(0.5));
    let client = app.playback_client_mut().unwrap();
    let (elapsed, _) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(90.0));
}

#[test]
fn app_progress_seek_noop_without_duration() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ProgressSeek(0.5));
    let client = app.playback_client_mut().unwrap();
    let (elapsed, duration) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, None);
    assert_eq!(duration, None);
}

#[test]
fn app_playback_state_update_tracks_position() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
        ],
    );
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.playback_client_mut().unwrap().play_album(0);
    let _ = app.playback_client_mut().unwrap().switch_to_track(0, 1);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::PlaybackStateUpdate {
        state: State::Play,
        current_song_position: Some(1),
        elapsed_secs: Some(10.0),
        duration_secs: Some(200.0),
    });

    let mw = app.main_window().unwrap();
    assert!(mw.is_track_current(0, 1));
}


#[test]
fn app_dismiss_error_clears_error() {
    use plinth::mpd::playback::PlaybackClientError;
    use plinth::ui::AppState;

    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    if let AppState::Running { error, .. } = &mut app.state {
        *error = Some(PlaybackClientError::MPDRetrieveError("boom".into()));
    } else {
        panic!("Expected Running state");
    }

    let _task = app.update(Message::DismissError);

    match &app.state {
        AppState::Running { error, .. } => assert!(error.is_none()),
        _ => panic!(),
    }
}

#[test]
fn app_grid_album_click_first_album_plays() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let task = app.update(Message::GridAlbumClicked(0));
    for msg in extract_task_messages(task) {
        let _ = app.update(msg);
    }
    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 1);
    assert_eq!(mw.playback_state(), State::Play);
}


#[test]
fn app_grid_click_then_now_playing_shows_album() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1, album2], backend);

    assert_eq!(app.main_window().unwrap().playing_album_count(), 0);

    let _ = update_and_replay(&mut app, Message::GridAlbumClicked(0));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 1);

    let _ = update_and_replay(&mut app, Message::GridAlbumClicked(1));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 2);

    let _ = app.update(Message::NowPlayingButtonClicked);
    let mw = app.main_window().unwrap();
    assert_eq!(mw.current_view(), plinth::ui::main_window::View::NowPlaying);
    assert_eq!(mw.playing_album_count(), 2);
}


#[test]
fn app_grid_click_replaces_queue_on_first_play() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1.clone(), album2.clone()], backend);

    let _ = update_and_replay(&mut app, Message::GridAlbumClicked(0));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 1);

    let _ = update_and_replay(&mut app, Message::GridAlbumClicked(1));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 2);

    let _ = app.playback_client_mut().unwrap().queue.borrow_mut().clear();
    let _ = update_and_replay(&mut app, Message::GridAlbumClicked(1));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 1);
}