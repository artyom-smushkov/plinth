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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use mpd::{Client, State};

use crate::mpd::playback::PlaybackClientError;
use crate::ui::types::Message;

static RELOAD_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub struct ReloadFlag;

impl ReloadFlag {
    pub fn new() -> Self {
        RELOAD_FLAG.set(Arc::new(AtomicBool::new(false))).ok();
        Self
    }

    pub fn clone_handle(&self) -> ReloadFlag {
        Self
    }

    pub fn set(&self, value: bool) {
        if let Some(flag) = RELOAD_FLAG.get() {
            flag.store(value, Ordering::Relaxed);
        }
    }

    fn is_set() -> bool {
        RELOAD_FLAG.get().map_or(false, |f| f.load(Ordering::Relaxed))
    }

    fn clear() {
        if let Some(flag) = RELOAD_FLAG.get() {
            flag.store(false, Ordering::Relaxed);
        }
    }
}

pub struct Poller {
    connection: MpdPollConnection,
    sender: tokio::sync::mpsc::Sender<Message>,
    last_state: State,
    last_elapsed: Option<f64>,
    last_duration: Option<f64>,
    last_song_pos: Option<usize>,
    last_updating_db: Option<u32>,
}

impl Poller {
    pub fn new(addr: &str, sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self {
            connection: MpdPollConnection::new(addr),
            sender,
            last_state: State::Stop,
            last_elapsed: None,
            last_duration: None,
            last_song_pos: None,
            last_updating_db: None,
        }
    }

    pub async fn run(mut self) {
        loop {
            match self.poll_status() {
                Ok(msgs) => {
                    for msg in msgs {
                        let _ = self.sender.send(msg).await;
                    }
                }
                Err(_) => {
                    let _ = self.connection.reconnect();
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

     fn poll_status(&mut self) -> Result<Vec<Message>, PlaybackClientError> {
        let status = self.connection.get_status()?;
        let mut messages = Vec::new();

        let updating_db = status.updating_db;
        if updating_db != self.last_updating_db {
            messages.push(Message::UpdatingDatabase(updating_db));
        }
        self.last_updating_db = updating_db;

        if ReloadFlag::is_set() && updating_db.is_none() {
            ReloadFlag::clear();
            messages.push(Message::DatabaseUpdated);
        }

        let state = status.state;
        let elapsed = status.elapsed.map(|d| d.as_secs_f64());
        let duration = status.duration.map(|d| d.as_secs_f64());
        let song_pos = status.song.map(|s| s.pos as usize);

        let state_changed = state != self.last_state
            || elapsed != self.last_elapsed
            || duration != self.last_duration
            || song_pos != self.last_song_pos;

        if !state_changed && messages.is_empty() {
            return Ok(messages);
        }

        if state_changed {
            self.last_state = state;
            self.last_elapsed = elapsed;
            self.last_duration = duration;
            self.last_song_pos = song_pos;
            messages.push(Message::SetPlaybackState(state, elapsed, duration, song_pos));
        }

        Ok(messages)
    }
}

struct MpdPollConnection {
    client: Option<Client>,
    addr: String,
    connected_at: Option<Instant>,
}

impl MpdPollConnection {
    fn new(addr: &str) -> Self {
        Self {
            client: None,
            addr: addr.to_string(),
            connected_at: None,
        }
    }

    fn get_status(&mut self) -> Result<mpd::Status, PlaybackClientError> {
        if self.client.is_none() {
            self.reconnect()?;
        }

        let result = self.client.as_mut().unwrap().status().map_err(|e| {
            PlaybackClientError::MPDRetrieveError(format!("Failed to get status: {}", e))
        });

        if result.is_err() {
            self.client = None;
            self.reconnect()?;
            return self.client.as_mut().unwrap().status().map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to get status: {}", e))
            });
        }

        result
    }

    fn reconnect(&mut self) -> Result<(), PlaybackClientError> {
        let client = Client::connect(&self.addr).map_err(|e| {
            PlaybackClientError::MPDConnectionError(format!("MPD connection failed: {}", e))
        })?;
        self.client = Some(client);
        self.connected_at = Some(Instant::now());
        Ok(())
    }
}

pub fn polling_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run(|| {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let poller = Poller::new("127.0.0.1:6600", sender);
            poller.run().await;
        });

        async_stream::stream! {
            while let Some(msg) = receiver.recv().await {
                yield msg;
            }
        }
    })
}