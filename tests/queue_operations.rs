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
fn play_album_clears_queue() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();

    let mut client = make_client(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    assert_eq!(*client.queue.borrow(), vec![0]);

    let _ = client.play_album(1);
    assert_eq!(*client.queue.borrow(), vec![1]);
}

#[test]
fn play_album_sets_cursor() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let _ = client.play_album(0);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 0);
    assert_eq!(cursor.track_index, 0);
    assert_eq!(cursor.position_ms, 0);
}

#[test]
fn play_album_sets_playback_state() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let _ = client.play_album(0);

    let state = client.playback_state().unwrap();
    assert_eq!(state, mpd::State::Play);
}

#[test]
fn play_album_invalid_index_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let result = client.play_album(99);
    assert!(result.is_err());
}

#[test]
fn play_album_empty_album_error() {
    let album = make_album("Empty Album", "Artist", vec![]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let result = client.play_album(0);
    assert!(result.is_err());
}

#[test]
fn add_album_empty_queue_plays() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let result = client.add_album_to_queue(0);
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
    let state = client.playback_state().unwrap();
    assert_eq!(state, mpd::State::Play);
}

#[test]
fn add_album_appends_to_queue() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    let result = client.add_album_to_queue(1);
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}

#[test]
fn add_album_duplicate_noop() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1.clone(), album2.clone()], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 1]);

    let result = client.add_album_to_queue(1);
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}

#[test]
fn add_album_invalid_index_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone()], backend);

    let result = client.add_album_to_queue(99);
    assert!(result.is_err());
}

#[test]
fn album_index_valid_rejects_empty_albums() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let empty_album = make_album("Empty", "Artist", vec![]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album.clone(), empty_album.clone()], backend);

    assert!(client.play_album(0).is_ok());
    assert!(client.play_album(1).is_err());
    assert!(client.play_album(2).is_err());
}

#[test]
fn clear_queue_resets_everything() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
    assert!(client.cursor.is_some());

    let _ = client.clear_queue();
    assert!(client.queue.borrow().is_empty());
    assert!(client.cursor.is_none());
}

#[test]
fn clear_queue_from_empty_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let result = client.clear_queue();
    assert!(result.is_ok());
    assert!(client.queue.borrow().is_empty());
    assert!(client.cursor.is_none());
}

#[test]
fn play_album_pushes_all_tracks_to_backend() {
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
    assert_eq!(*client.queue.borrow(), vec![0]);
}

#[test]
fn add_album_to_queue_pushes_tracks_to_backend() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![
        make_track("T2", make_song(2, "T2", 200)),
        make_track("T3", make_song(3, "T3", 220)),
    ]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}