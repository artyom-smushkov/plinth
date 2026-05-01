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

#[test]
fn toggle_play_pause_from_stop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let state = client.playback_state().unwrap();
    assert_eq!(state, State::Stop);

    let _ = client.toggle_play_pause();
    let state = client.playback_state().unwrap();
    assert_eq!(state, State::Stop);
}

#[test]
fn toggle_play_pause_from_play() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_state(State::Play);
    let mut client = make_client(vec![album], backend);

    let _ = client.toggle_play_pause();
    let state = client.playback_state().unwrap();
    assert_eq!(state, State::Pause);
}

#[test]
fn toggle_play_pause_from_pause() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_state(State::Pause);
    let mut client = make_client(vec![album], backend);

    let _ = client.toggle_play_pause();
    let state = client.playback_state().unwrap();
    assert_eq!(state, State::Play);
}

#[test]
fn seek_to_updates_elapsed() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(0.0, 180.0);
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);
    let _ = client.seek_to(45.0);

    let (elapsed, duration) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(45.0));
    assert_eq!(duration, Some(180.0));
}

#[test]
fn previous_song_decrements_position() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
            make_track("T3", make_song(3, "T3", 220)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);
    let _ = client.switch_to_track(0, 2);

    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 2);

    let _ = client.previous_song();
    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 1);
}

#[test]
fn next_song_increments_position() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
            make_track("T3", make_song(3, "T3", 220)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);

    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 0);

    let _ = client.next_song();
    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 1);
}

#[test]
fn prev_at_start_stays_at_zero() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);
    let _ = client.previous_song();

    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 0);
}

#[test]
fn next_at_end_stays_at_last() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);
    let _ = client.switch_to_track(0, 1);

    let _ = client.next_song();
    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 1);
}

#[test]
fn get_full_status_returns_all_fields() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::with_elapsed_duration(45.0, 180.0);
    let mut client = make_client(vec![album], backend);

    let _ = client.play_album(0);
    let (state, elapsed, duration, song_pos) = client.get_full_status().unwrap();
    assert_eq!(state, mpd::status::State::Play);
    assert_eq!(elapsed, Some(0.0));
    assert_eq!(duration, Some(180.0));
    assert_eq!(song_pos, Some(0));
}

#[test]
fn get_full_status_none_values() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let (state, elapsed, duration, song_pos) = client.get_full_status().unwrap();
    assert_eq!(state, mpd::status::State::Stop);
    assert_eq!(elapsed, None);
    assert_eq!(duration, None);
    assert_eq!(song_pos, None);
}

#[test]
fn seek_to_with_no_song_pos_defaults_to_zero() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let _ = client.seek_to(30.0);
    let (elapsed, _) = client.get_elapsed_duration().unwrap();
    assert_eq!(elapsed, Some(30.0));
}