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
use mpd::song::Song;
use plinth::ui::types::Message;
use std::time::Duration;

fn make_song_with_file(_id: u32, file: &str, title: &str, duration_secs: u64) -> Song {
    Song {
        file: file.to_string(),
        title: Some(title.to_string()),
        artist: Some("Test Artist".to_string()),
        duration: Some(Duration::from_secs(duration_secs)),
        tags: vec![(String::from("Album"), String::from("Test Album"))],
        ..Default::default()
    }
}

#[test]
fn init_queue_empty_mpd_noop() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert!(client.queue.borrow().is_empty());
}

#[test]
fn init_queue_single_album_syncs() {
    let song1 = make_song_with_file(1, "/music/a1.mp3", "T1", 180);
    let song2 = make_song_with_file(2, "/music/a2.mp3", "T2", 200);
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", song1.clone()),
            make_track("T2", song2.clone()),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![song1, song2]);
    let mut client = make_client(vec![album], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
}

#[test]
fn init_queue_multiple_albums_in_order_syncs() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
    ]);
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}

#[test]
fn init_queue_interleaved_rebuilds() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180)),
            make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
        make_song_with_file(3, "/music/a3.mp3", "T3", 220),
    ]);
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}

#[test]
fn init_queue_unknown_track_rebuilds() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(99, "/music/unknown.mp3", "Unknown", 100),
    ]);
    let mut client = make_client(vec![album1], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
}

#[test]
fn init_queue_duplicate_albums_rebuilds() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180)),
            make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
        make_song_with_file(3, "/music/a3.mp3", "T3", 220),
    ]);
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
}

#[test]
fn init_queue_restores_cursor_when_playing() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![
            make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200)),
            make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220)),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
        make_song_with_file(3, "/music/a3.mp3", "T3", 220),
    ]);
    backend.current_pos = 2;
    backend.state = mpd::status::State::Play;
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 1);
    assert_eq!(cursor.track_index, 1);
}

#[test]
fn init_queue_rebuild_preserves_album_order() {
    let album1 = make_album(
        "Album A",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album B",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let album3 = make_album(
        "Album C",
        "Artist",
        vec![make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(3, "/music/a3.mp3", "T3", 220),
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
    ]);
    let mut client = make_client(vec![album1, album2, album3], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![2, 0, 1]);
}

#[test]
fn app_init_syncs_queue_to_ui_when_non_empty() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
    ]);
    let client = make_client(vec![album1, album2], backend);
    *client.queue.borrow_mut() = vec![0, 1];

    let (app, task) = app_with_test_client(client);
    let messages = extract_task_messages(task);

    assert!(
        messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "App init task should contain SyncQueue when queue is non-empty"
    );

    let mut app = app;
    for msg in messages {
        let _ = app.update(msg);
    }

    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 2);
}

#[test]
fn app_init_no_sync_queue_when_empty() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let client = make_client(vec![album], backend);

    let (_app, task) = app_with_test_client(client);
    let messages = extract_task_messages(task);

    assert!(
        !messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "App init task should NOT contain SyncQueue when queue is empty"
    );
}

#[test]
fn app_init_queue_populates_now_playing() {
    let song1 = make_song_with_file(1, "/music/a1.mp3", "T1", 180);
    let song2 = make_song_with_file(2, "/music/a2.mp3", "T2", 200);
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", song1.clone()),
            make_track("T2", song2.clone()),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![song1, song2]);
    let client = make_client(vec![album], backend);
    client.queue.borrow_mut().push(0);

    let (app, task) = app_with_test_client(client);
    let messages = extract_task_messages(task);

    let mut app = app;
    for msg in messages {
        let task = app.update(msg);
        let replay = extract_task_messages(task);
        for m in replay {
            let _ = app.update(m);
        }
    }

    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 1);
}

#[test]
fn divergence_position_mismatch_triggers_rebuild() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![
        make_song_with_file(1, "/music/a1.mp3", "T1", 180),
        make_song_with_file(99, "/music/external.mp3", "External", 100),
        make_song_with_file(2, "/music/a2.mp3", "T2", 200),
    ]);
    backend.current_pos = 2;
    backend.state = mpd::status::State::Play;
    let mut client = make_client(vec![album1, album2], backend);
    client.queue.borrow_mut().extend([0, 1]);

    let (app, _init_task) = app_with_test_client(client);

    let mut app = app;

    let task = app.update(Message::PlaybackStateUpdate {
        state: mpd::status::State::Play,
        current_song_position: Some(0),
        elapsed_secs: Some(0.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);

    assert!(
        messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "Position mismatch should trigger queue rebuild"
    );
}

 #[test]
fn divergence_queue_cleared_externally_triggers_rebuild() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180))],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200))],
    );
    let mut backend = MockBackend::with_state(mpd::status::State::Pause);
    let mut client = make_client(vec![album1, album2], backend);
    client.queue.borrow_mut().extend([0, 1]);

    let (app, _init_task) = app_with_test_client(client);

    let mut app = app;

    let task = app.update(Message::PlaybackStateUpdate {
        state: mpd::status::State::Play,
        current_song_position: Some(0),
        elapsed_secs: Some(0.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);

    assert!(
        messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "Queue cleared externally should trigger rebuild"
    );
}

#[test]
fn no_divergence_when_stopped_and_empty() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let client = make_client(vec![album], backend);

    let (app, _init_task) = app_with_test_client(client);

    let mut app = app;

    let task = app.update(Message::PlaybackStateUpdate {
        state: mpd::status::State::Stop,
        current_song_position: None,
        elapsed_secs: None,
        duration_secs: None,
    });
    let messages = extract_task_messages(task);

    assert!(
        !messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "Stopped state with empty queue should not trigger rebuild"
    );
}

#[test]
fn no_divergence_when_position_matches() {
    let song1 = make_song_with_file(1, "/music/a1.mp3", "T1", 180);
    let song2 = make_song_with_file(2, "/music/a2.mp3", "T2", 200);
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", song1.clone()),
            make_track("T2", song2.clone()),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.extend(vec![song1, song2]);
    backend.current_pos = 1;
    backend.state = mpd::status::State::Play;
    let mut client = make_client(vec![album], backend);
    client.queue.borrow_mut().push(0);

    let (app, _init_task) = app_with_test_client(client);

    let mut app = app;

    let task = app.update(Message::PlaybackStateUpdate {
        state: mpd::status::State::Play,
        current_song_position: Some(0),
        elapsed_secs: Some(0.0),
        duration_secs: Some(180.0),
    });
    let first_messages = extract_task_messages(task);

    let task = app.update(Message::PlaybackStateUpdate {
        state: mpd::status::State::Play,
        current_song_position: Some(1),
        elapsed_secs: Some(10.0),
        duration_secs: Some(180.0),
    });
    let messages = extract_task_messages(task);

    assert!(
        !messages.iter().any(|m| matches!(m, Message::SyncQueue(_))),
        "Matching position should not trigger rebuild"
    );
    assert!(
        first_messages.iter().all(|m| !matches!(m, Message::SyncQueue(_))),
        "First update should also not trigger rebuild"
    );
}

#[test]
fn init_queue_partial_album_triggers_rebuild() {
    let song1 = make_song_with_file(1, "/music/a1.mp3", "T1", 180);
    let song2 = make_song_with_file(2, "/music/a2.mp3", "T2", 200);
    let song3 = make_song_with_file(3, "/music/a3.mp3", "T3", 220);
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", song1.clone()),
            make_track("T2", song2.clone()),
            make_track("T3", song3.clone()),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.push(song2.clone());
    let mut client = make_client(vec![album], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
    let mpd_queue = client.backend.queue().unwrap();
    assert_eq!(mpd_queue.len(), 3);
    assert_eq!(mpd_queue[0].file, song1.file);
    assert_eq!(mpd_queue[1].file, song2.file);
    assert_eq!(mpd_queue[2].file, song3.file);
}

#[test]
fn init_queue_partial_album_middle_tracks_triggers_rebuild() {
    let song1 = make_song_with_file(1, "/music/a1.mp3", "T1", 180);
    let song2 = make_song_with_file(2, "/music/a2.mp3", "T2", 200);
    let song3 = make_song_with_file(3, "/music/a3.mp3", "T3", 220);
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", song1.clone()),
            make_track("T2", song2.clone()),
            make_track("T3", song3.clone()),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.push(song2.clone());
    backend.queue.push(song3.clone());
    let mut client = make_client(vec![album], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
    let mpd_queue = client.backend.queue().unwrap();
    assert_eq!(mpd_queue.len(), 3);
}

#[test]
fn init_queue_partial_album_then_complete_album_rebuilds() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180)),
            make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220))],
    );
    let mut backend = MockBackend::new();
    backend.queue.push(make_song_with_file(1, "/music/a1.mp3", "T1", 180));
    backend.queue.push(make_song_with_file(3, "/music/a3.mp3", "T3", 220));
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0, 1]);
    let mpd_queue = client.backend.queue().unwrap();
    assert_eq!(mpd_queue.len(), 3);
}

#[test]
fn init_queue_rebuild_restores_cursor_when_playing() {
    let album1 = make_album(
        "Album 1",
        "Artist",
        vec![
            make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180)),
            make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200)),
        ],
    );
    let album2 = make_album(
        "Album 2",
        "Artist",
        vec![
            make_track("T3", make_song_with_file(3, "/music/a3.mp3", "T3", 220)),
            make_track("T4", make_song_with_file(4, "/music/a4.mp3", "T4", 240)),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.push(make_song_with_file(1, "/music/a1.mp3", "T1", 180));
    backend.queue.push(make_song_with_file(3, "/music/a3.mp3", "T3", 220));
    backend.queue.push(make_song_with_file(4, "/music/a4.mp3", "T4", 240));
    backend.current_pos = 2;
    backend.state = mpd::status::State::Play;
    let mut client = make_client(vec![album1, album2], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    let cursor = client.cursor.as_ref().unwrap();
    assert_eq!(cursor.queued_album_index, 1);
    assert_eq!(cursor.track_index, 0);
}

#[test]
fn init_queue_partial_album_with_unknown_track_rebuilds() {
    let album = make_album(
        "Album",
        "Artist",
        vec![
            make_track("T1", make_song_with_file(1, "/music/a1.mp3", "T1", 180)),
            make_track("T2", make_song_with_file(2, "/music/a2.mp3", "T2", 200)),
        ],
    );
    let mut backend = MockBackend::new();
    backend.queue.push(make_song_with_file(1, "/music/a1.mp3", "T1", 180));
    backend.queue.push(make_song_with_file(99, "/music/unknown.mp3", "Unknown", 100));
    let mut client = make_client(vec![album], backend);

    let result = client.init_queue_from_mpd();
    assert!(result.is_ok());
    assert_eq!(*client.queue.borrow(), vec![0]);
    let mpd_queue = client.backend.queue().unwrap();
    assert_eq!(mpd_queue.len(), 2);
}