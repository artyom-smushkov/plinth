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

use plinth::mpd::types::Album;
use plinth::ui::main_window::View;
use plinth::ui::types::Message;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn track_widget_hover_toggle() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::TrackHovered(0, 0));
    let mw = app.main_window().unwrap();
    assert!(mw.is_track_hovered(0, 0));

    let _ = app.update(Message::TrackUnhovered(0, 0));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_track_hovered(0, 0));
}

#[test]
fn track_widget_track_ended_playing() {
    let album = make_album("Album", "Artist", vec![
        make_track("T1", make_song(1, "T1", 180)),
        make_track("T2", make_song(2, "T2", 200)),
    ]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::SetCurrentTrack(0, 0));
    let mw = app.main_window().unwrap();
    assert!(mw.is_track_current(0, 0));

    let _ = app.update(Message::SetCurrentTrack(0, 1));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_track_current(0, 0));
    assert!(mw.is_track_current(0, 1));
}

#[test]
fn track_widget_ignores_wrong_indices() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::TrackHovered(1, 0));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_track_hovered(0, 0));

    let _ = app.update(Message::SetCurrentTrack(1, 1));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_track_current(0, 0));
}

#[test]
fn main_window_switch_to_album_grid() {
    let albums: Vec<Rc<Album>> = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::NowPlayingButtonClicked);
    let mw = app.main_window().unwrap();
    assert_eq!(mw.current_view(), View::NowPlaying);

    let _ = app.update(Message::AlbumGridButtonClicked);
    let mw = app.main_window().unwrap();
    assert_eq!(mw.current_view(), View::AlbumGrid);
}

#[test]
fn main_window_default_view_is_album_grid() {
    let albums: Vec<Rc<Album>> = vec![];
    let backend = MockBackend::new();
    let app = make_app(albums, backend);

    let mw = app.main_window().unwrap();
    assert_eq!(mw.current_view(), View::AlbumGrid);
}

#[test]
fn main_window_set_playback_state() {
    let albums: Vec<Rc<Album>> = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SetPlaybackState(
        mpd::State::Play,
        Some(45.0),
        Some(180.0),
        None,
    ));

    let mw = app.main_window().unwrap();
    assert_eq!(mw.playback_state(), mpd::State::Play);
    assert_eq!(mw.elapsed_secs(), Some(45.0));
    assert_eq!(mw.duration_secs(), Some(180.0));
}

#[test]
fn main_window_initial_state_is_stop() {
    let albums: Vec<Rc<Album>> = vec![];
    let backend = MockBackend::new();
    let app = make_app(albums, backend);

    let mw = app.main_window().unwrap();
    assert_eq!(mw.playback_state(), mpd::State::Stop);
}

#[test]
fn now_playing_sync_queue() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1, album2], backend);

    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 0);

    let queue = RefCell::new(vec![0, 1]);
    let _ = app.update(Message::SyncQueue(queue));
    let mw = app.main_window().unwrap();
    assert_eq!(mw.playing_album_count(), 2);
}

#[test]
fn album_hovered_sets_hover_state() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::AlbumHovered(0));
    let mw = app.main_window().unwrap();
    assert!(mw.is_album_hovered(0));
}

#[test]
fn album_unhovered_clears_hover_state() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::AlbumHovered(0));
    let mw = app.main_window().unwrap();
    assert!(mw.is_album_hovered(0));

    let _ = app.update(Message::AlbumUnhovered(0));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_album_hovered(0));
}

#[test]
fn set_current_track_updates_all_track_widgets() {
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
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let _ = app.update(Message::SetCurrentTrack(0, 1));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_track_current(0, 0));
    assert!(mw.is_track_current(0, 1));
    assert!(!mw.is_track_current(0, 2));
}

#[test]
fn now_playing_album_thumbnail_ready() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let result: Result<(String, Vec<u8>), (String, plinth::mpd::playback::PlaybackClientError)> =
        Ok(("now-playing-Artist-Album".to_string(), vec![0xFF, 0xD8, 0xFF]));
    let _ = app.update(Message::NowPlayingThumbnailReady(
        "now-playing-Artist-Album".to_string(),
        result,
    ));
    let mw = app.main_window().unwrap();
    assert!(mw.is_thumbnail_loaded(0));
}

#[test]
fn now_playing_album_thumbnail_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let result: Result<(String, Vec<u8>), (String, plinth::mpd::playback::PlaybackClientError)> =
        Err(("now-playing-Artist-Album".to_string(), plinth::mpd::playback::PlaybackClientError::MPDRetrieveError("test error".into())));
    let _ = app.update(Message::NowPlayingThumbnailReady(
        "now-playing-Artist-Album".to_string(),
        result,
    ));
    let mw = app.main_window().unwrap();
    assert!(!mw.is_thumbnail_loaded(0));
}

#[test]
fn now_playing_remove_album_from_queue() {
    let album1 = make_album("Album 1", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1, album2], backend);

    let client = app.playback_client_mut().unwrap();
    client.queue.borrow_mut().push(0);
    client.queue.borrow_mut().push(1);

    let queue = RefCell::new(vec![0, 1]);
    let _ = app.update(Message::SyncQueue(queue));
    assert_eq!(app.main_window().unwrap().playing_album_count(), 2);

    let _ = app.update(Message::RemoveAlbumFromQueue(0));
    let client = app.playback_client_mut().unwrap();
    assert_eq!(*client.queue.borrow(), vec![1]);
}