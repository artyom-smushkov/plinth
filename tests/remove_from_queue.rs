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

use plinth::mpd::types::PlaybackCursor;

#[test]
fn remove_first_of_multiple_adjusts_cursor() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let album3 = make_album("Album 3", "Artist", vec![make_track("T3", make_song(3, "T3", 220))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2, album3], backend);

    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);
    client.queue.borrow_mut().push(2);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 2,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(0);
    assert_eq!(*client.queue.borrow(), vec![1, 2]);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 1);
}

#[test]
fn remove_current_album_clears_cursor() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2], backend);

    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 1,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0]);
    assert!(client.cursor.is_none());
}

#[test]
fn remove_middle_album_adjusts_cursor() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let album3 = make_album("Album 3", "Artist", vec![make_track("T3", make_song(3, "T3", 220))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2, album3], backend);

    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);
    client.queue.borrow_mut().push(2);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 2,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 2]);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 1);
}

#[test]
fn remove_middle_album_cursor_before_unchanged() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let album3 = make_album("Album 3", "Artist", vec![make_track("T3", make_song(3, "T3", 220))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2, album3], backend);

    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);
    client.queue.borrow_mut().push(2);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 0,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 2]);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 0);
}

#[test]
fn remove_last_album_no_cursor_adjust() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2], backend);

    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 0,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0]);

    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 0);
}

#[test]
fn remove_invalid_index_returns_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    client.queue.borrow_mut().push(0);
    let result = client.remove_album_from_queue(99);
    assert!(result.is_err());
}

#[test]
fn remove_only_album_clears_queue() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    client.queue.borrow_mut().push(0);
    client.cursor = Some(std::rc::Rc::new(PlaybackCursor {
        queued_album_index: 0,
        track_index: 0,
        position_ms: 0,
    }));

    let _ = client.remove_album_from_queue(0);
    assert!(client.queue.borrow().is_empty());
    assert!(client.cursor.is_none());
}

#[test]
fn remove_album_deletes_correct_mpd_range() {
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
    let album3 = make_album(
        "Album 3",
        "Artist",
        vec![
            make_track("C1", make_song(6, "C1", 100)),
            make_track("C2", make_song(7, "C2", 100)),
        ],
    );
    let backend = MockBackend::new();
    let mut client = make_client(vec![album1, album2, album3], backend);

    let _ = client.play_album(0);
    let _ = client.add_album_to_queue(1);
    let _ = client.add_album_to_queue(2);
    assert_eq!(*client.queue.borrow(), vec![0, 1, 2]);

    let _ = client.remove_album_from_queue(1);
    assert_eq!(*client.queue.borrow(), vec![0, 2]);
}