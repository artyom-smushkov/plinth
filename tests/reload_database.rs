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
use plinth::ui::AppState;
use plinth::ui::types::Message;
use std::rc::Rc;

#[test]
fn reload_database_calls_backend() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let result = client.reload_database();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn reload_database_returns_incrementing_job_id() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut client = make_client(vec![album], backend);

    let job1 = client.reload_database().unwrap();
    let job2 = client.reload_database().unwrap();

    assert_eq!(job1, 1);
    assert_eq!(job2, 2);
}

#[test]
fn reload_database_failing_backend_returns_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["reload_database"]);
    let mut client = make_client_with(vec![album], backend);

    let result = client.reload_database();
    assert!(result.is_err());
}

#[test]
fn reload_database_noop_when_updating() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let (mut app, _) = app_with_test_client(make_client(vec![album], backend));

    let task1 = app.update(Message::ReloadDatabase);
    let msgs1 = extract_task_messages(task1);
    assert!(msgs1.is_empty());

    let _ = app.update(Message::UpdatingDatabase(Some(1)));

    let task2 = app.update(Message::ReloadDatabase);
    let msgs2 = extract_task_messages(task2);
    assert!(msgs2.is_empty());

    if let AppState::Running { updating_db, .. } = &app.state {
        assert_eq!(updating_db, &Some(1));
    } else {
        panic!("Expected Running state");
    }
}

#[test]
fn reload_database_clears_pending_when_update_ends_without_reload() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let (mut app, _) = app_with_test_client(make_client(vec![album], backend));

    let task = app.update(Message::ReloadDatabase);
    let msgs = extract_task_messages(task);
    assert!(msgs.is_empty());

    if let AppState::Running { pending_reload, .. } = &app.state {
        assert!(*pending_reload);
    } else {
        panic!("Expected Running state");
    }

    let _ = app.update(Message::UpdatingDatabase(None));

    if let AppState::Running { pending_reload, updating_db, .. } = &app.state {
        assert!(!*pending_reload);
        assert_eq!(updating_db, &None);
    } else {
        panic!("Expected Running state");
    }
}

#[test]
fn reload_database_error_sets_app_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = FailingMockBackend::failing_on(&["reload_database"]);
    let mut app = make_app_with(vec![album], backend);

    let _ = app.update(Message::ReloadDatabase);

    if let AppState::Running { error, pending_reload, .. } = &app.state {
        assert!(error.is_some());
        assert!(!*pending_reload);
    } else {
        panic!("Expected Running state");
    }
}

#[test]
fn database_updated_success_clears_pending_and_reloads() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let (mut app, _) = app_with_test_client(make_client(vec![album.clone()], backend));

    if let AppState::Running { playback_client, .. } = &mut app.state {
        playback_client.set_test_reload_result(Ok(Rc::new(vec![album])));
    } else {
        panic!("Expected Running state");
    }

    let _ = app.update(Message::ReloadDatabase);

    if let AppState::Running { pending_reload, .. } = &app.state {
        assert!(*pending_reload);
    } else {
        panic!("Expected Running state");
    }

    let _ = app.update(Message::DatabaseUpdated);

    if let AppState::Running { pending_reload, error, .. } = &app.state {
        assert!(!*pending_reload);
        assert!(error.is_none());
    } else {
        panic!("Expected Running state");
    }
}

#[test]
fn database_updated_error_sets_app_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let (mut app, _) = app_with_test_client(make_client(vec![album], backend));

    if let AppState::Running { playback_client, .. } = &mut app.state {
        playback_client.set_test_reload_result(Err(PlaybackClientError::MPDRetrieveError("test reload failure".into())));
    } else {
        panic!("Expected Running state");
    }

    let _ = app.update(Message::DatabaseUpdated);

    if let AppState::Running { pending_reload, error, .. } = &app.state {
        assert!(!*pending_reload);
        assert!(error.is_some());
    } else {
        panic!("Expected Running state");
    }
}