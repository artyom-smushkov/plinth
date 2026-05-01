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
fn switch_first_album_first_track() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let _ = client.play_album(0);
    let result = client.switch_to_track(0, 0);

    assert!(result.is_ok());
    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 0);
    assert_eq!(cursor.track_index, 0);
}

#[test]
fn switch_second_album_third_track() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("A1", make_song(1, "A1", 100)),
            make_track("A2", make_song(2, "A2", 100)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![
            make_track("B1", make_song(3, "B1", 100)),
            make_track("B2", make_song(4, "B2", 100)),
            make_track("B3", make_song(5, "B3", 100)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);

    let result = client.switch_to_track(1, 2);
    assert!(result.is_ok());

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 1);
    assert_eq!(cursor.track_index, 2);
}

#[test]
fn switch_calculates_correct_mpd_position() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("A1", make_song(1, "A1", 100)),
            make_track("A2", make_song(2, "A2", 100)),
            make_track("A3", make_song(3, "A3", 100)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![
            make_track("B1", make_song(4, "B1", 100)),
            make_track("B2", make_song(5, "B2", 100)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);

    let _ = client.switch_to_track(1, 1);

    let pos = client.current_song_position().unwrap().unwrap();
    assert_eq!(pos, 4);
}

#[test]
fn switch_out_of_bounds_queue_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let result = client.switch_to_track(99, 0);
    assert!(result.is_ok());
}

#[test]
fn switch_out_of_bounds_track_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let _ = client.play_album(0);
    let result = client.switch_to_track(0, 99);
    assert!(result.is_ok());
}

#[test]
fn cursor_resets_position_ms() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song(1, "T1", 180)),
            make_track("T2", make_song(2, "T2", 200)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let _ = client.play_album(0);
    let _ = client.switch_to_track(0, 1);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.position_ms, 0);
}