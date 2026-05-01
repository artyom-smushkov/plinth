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

#[test]
fn play_album_fails_on_clear_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["clear"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.play_album(0);
    assert!(result.is_err());
}

#[test]
fn play_album_fails_on_push_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["push"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.play_album(0);
    assert!(result.is_err());
}

#[test]
fn play_album_fails_on_switch_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["switch"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.play_album(0);
    assert!(result.is_err());
}

#[test]
fn seek_to_fails_on_status_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["status"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.seek_to(45.0);
    assert!(result.is_err());
}

#[test]
fn seek_to_fails_on_seek_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::with_elapsed_duration(0.0, 180.0, &["seek"]);
    let mut client = make_client_with(vec![album], backend);

    let _ = client.play_album(0);
    let result = client.seek_to(45.0);
    assert!(result.is_err());
}

#[test]
fn toggle_play_pause_propagates_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::with_state(mpd::status::State::Play, &["toggle_pause"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.toggle_play_pause();
    assert!(result.is_err());
}

#[test]
fn switch_to_track_fails_on_switch_error() {
    let album = make_album("Album", "Artist", vec![
        make_track("T1", make_song(1, "T1", 180)),
        make_track("T2", make_song(2, "T2", 200)),
    ]);
    let backend = FailingMockBackend::failing_on(&["switch"]);
    let mut client = make_client_with(vec![album], backend);

    client.queue.borrow_mut().push(0);
    let result = client.switch_to_track(0, 1);
    assert!(result.is_err());
}

#[test]
fn previous_song_propagates_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["prev"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.previous_song();
    assert!(result.is_err());
}

#[test]
fn next_song_propagates_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["next"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.next_song();
    assert!(result.is_err());
}

#[test]
fn playback_state_fails_on_status_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["status"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.playback_state();
    assert!(result.is_err());
}

#[test]
fn get_elapsed_duration_fails_on_status_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["status"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.get_elapsed_duration();
    assert!(result.is_err());
}

#[test]
fn current_song_position_fails_on_status_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["status"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.current_song_position();
    assert!(result.is_err());
}

#[test]
fn add_album_to_queue_fails_on_push_error() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = FailingMockBackend::failing_on(&["push"]);
    let mut client = make_client_with(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    let result = client.add_album_to_queue(1);
    assert!(result.is_err());
}