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

use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use futures::StreamExt;
use iced::Task;
use mpd::song::Song;
use mpd::status::State;

pub use plinth::mpd::backend::test::MockBackend;
use plinth::mpd::backend::MpdBackend;
use plinth::mpd::playback::{PlaybackClient, PlaybackClientError};
use plinth::mpd::types::{Album, Track};
use plinth::ui::App;
use plinth::ui::types::Message;

pub fn make_song(id: u32, title: &str, duration_secs: u64) -> Song {
    Song {
        file: format!("/music/{}.mp3", id),
        title: Some(title.to_string()),
        artist: Some("Test Artist".to_string()),
        duration: Some(Duration::from_secs(duration_secs)),
        tags: vec![(String::from("Album"), String::from("Test Album"))],
        ..Default::default()
    }
}

pub fn make_track(title: &str, song: Song) -> Track {
    Track {
        title: title.to_string(),
        artist: "Test Artist".to_string(),
        cd_number: None,
        track_number: None,
        mpd_song: song,
    }
}

pub fn make_album(name: &str, artist: &str, tracks: Vec<Track>) -> Rc<Album> {
    Rc::new(Album {
        name: name.to_string(),
        artist: artist.to_string(),
        genre: "Rock".to_string(),
        date: Some("2024".to_string()),
        tracks,
    })
}

pub fn make_client(albums: Vec<Rc<Album>>, backend: MockBackend) -> PlaybackClient {
    PlaybackClient::new_for_test(Rc::new(albums), Box::new(backend))
}

pub fn make_client_with<B: MpdBackend + 'static>(albums: Vec<Rc<Album>>, backend: B) -> PlaybackClient {
    PlaybackClient::new_for_test(Rc::new(albums), Box::new(backend))
}

pub fn make_app(albums: Vec<Rc<Album>>, backend: MockBackend) -> App {
    let client = make_client(albums, backend);
    let (app, _task) = App::with_test_client(client);
    app
}

pub fn make_app_with<B: MpdBackend + 'static>(albums: Vec<Rc<Album>>, backend: B) -> App {
    let client = make_client_with(albums, backend);
    let (app, _task) = App::with_test_client(client);
    app
}

pub fn extract_task_messages(task: Task<Message>) -> Vec<Message> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut messages = Vec::new();
            if let Some(mut stream) = iced_runtime::task::into_stream(task) {
                while let Some(action) = stream.next().await {
                    if let iced_runtime::Action::Output(msg) = action {
                        messages.push(msg);
                    }
                }
            }
            messages
        })
}

pub fn update_and_replay(app: &mut App, message: Message) -> Vec<Message> {
    let task = app.update(message);
    let messages = extract_task_messages(task);
    for msg in &messages {
        let _ = app.update(msg.clone());
    }
    messages
}

pub struct FailingMockBackend {
    inner: MockBackend,
    fail_ops: HashSet<String>,
}

impl FailingMockBackend {
    pub fn failing_on(ops: &[&str]) -> Self {
        Self {
            inner: MockBackend::new(),
            fail_ops: ops.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn with_state(state: State, fail_ops: &[&str]) -> Self {
        Self {
            inner: MockBackend::with_state(state),
            fail_ops: fail_ops.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn with_elapsed_duration(elapsed: f64, duration: f64, fail_ops: &[&str]) -> Self {
        Self {
            inner: MockBackend::with_elapsed_duration(elapsed, duration),
            fail_ops: fail_ops.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl MpdBackend for FailingMockBackend {
    fn clear(&mut self) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("clear") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: clear".into()));
        }
        self.inner.clear()
    }

    fn push(&mut self, song: Song) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("push") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: push".into()));
        }
        self.inner.push(song)
    }

    fn delete_range(&mut self, start: u32, end: u32) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("delete_range") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: delete_range".into()));
        }
        self.inner.delete_range(start, end)
    }

    fn switch(&mut self, pos: u32) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("switch") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: switch".into()));
        }
        self.inner.switch(pos)
    }

    fn toggle_pause(&mut self) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("toggle_pause") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: toggle_pause".into()));
        }
        self.inner.toggle_pause()
    }

    fn status(&mut self) -> Result<mpd::Status, PlaybackClientError> {
        if self.fail_ops.contains("status") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: status".into()));
        }
        self.inner.status()
    }

    fn seek(&mut self, song_pos: u32, seconds: f64) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("seek") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: seek".into()));
        }
        self.inner.seek(song_pos, seconds)
    }

    fn prev(&mut self) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("prev") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: prev".into()));
        }
        self.inner.prev()
    }

    fn next(&mut self) -> Result<(), PlaybackClientError> {
        if self.fail_ops.contains("next") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: next".into()));
        }
        self.inner.next()
    }

    fn queue(&mut self) -> Result<Vec<Song>, PlaybackClientError> {
        if self.fail_ops.contains("queue") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: queue".into()));
        }
        self.inner.queue()
    }

    fn reload_database(&mut self) -> Result<u32, PlaybackClientError> {
        if self.fail_ops.contains("reload_database") {
            return Err(PlaybackClientError::MPDRetrieveError("mock failure: reload_database".into()));
        }
        self.inner.reload_database()
    }
}