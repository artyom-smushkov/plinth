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

use mpd::status::State;
use plinth::ui::types::Message;

#[test]
fn playback_state_update_noop_when_unchanged() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(10.0, 180.0);
    let mut app = make_app(vec![album], backend);

    let _ = update_and_replay(&mut app, Message::PlaybackStateUpdate {
        state: State::Stop,
        current_song_position: None,
        elapsed_secs: Some(10.0),
        duration_secs: Some(180.0),
    });

    let task = app.update(Message::PlaybackStateUpdate {
        state: State::Stop,
        current_song_position: None,
        elapsed_secs: Some(10.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);
    assert!(messages.is_empty());
}

#[test]
fn playback_state_update_propagates_on_first_call() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(10.0, 180.0);
    let mut app = make_app(vec![album], backend);

    let task = app.update(Message::PlaybackStateUpdate {
        state: State::Play,
        current_song_position: Some(0),
        elapsed_secs: Some(10.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);
    assert!(!messages.is_empty());
}

#[test]
fn progress_seek_noop_zero_duration() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(45.0, 0.0);
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ProgressSeek(0.5));
    let client = app.playback_client_mut().unwrap();
    let (elapsed, duration) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(45.0));
    assert_eq!(duration, Some(0.0));
}

#[test]
fn progress_seek_noop_zero_elapsed() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(0.0, 180.0);
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ProgressSeek(0.5));
    let client = app.playback_client_mut().unwrap();
    let (elapsed, duration) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(0.0));
    assert_eq!(duration, Some(180.0));
}

#[test]
fn track_click_invalid_queue_index_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let task = app.update(Message::TrackClicked(99, 0));
    let messages = extract_task_messages(task);
    assert_eq!(messages.len(), 1);
}

#[test]
fn remove_album_from_queue_invalid_index_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let task = app.update(Message::RemoveAlbumFromQueue(99));
    let msgs = extract_task_messages(task);
    for msg in &msgs {
        let _ = app.update(msg.clone());
    }
    let client = app.playback_client_mut().unwrap();
    assert!(client.queue.borrow().is_empty());
}

#[test]
fn playback_state_update_status_error_returns_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["status"]);
    let client = make_client_with(vec![album], backend);
    let (mut app, _task) = app_with_test_client(client);

    let task = app.update(Message::PlaybackStateUpdate {
        state: State::Play,
        current_song_position: Some(0),
        elapsed_secs: Some(10.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);
    assert!(messages.is_empty());
}

#[test]
fn clear_queue_resets_state() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.playback_client_mut().unwrap().play_album(0);

    let task = app.update(Message::ClearQueue);
    for msg in extract_task_messages(task) {
        let _ = app.update(msg);
    }

    let client = app.playback_client_mut().unwrap();
    assert!(client.queue.borrow().is_empty());
    assert!(client.cursor.is_none());
}

#[test]
fn play_pause_status_error_returns_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::with_state(State::Play, &["status"]);
    let client = make_client_with(vec![album], backend);
    let (mut app, _task) = app_with_test_client(client);

    let task = app.update(Message::PlayPause);
    let messages = extract_task_messages(task);
    assert!(messages.is_empty());
}