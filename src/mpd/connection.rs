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

use mpd::Client;
use std::time::{Duration, Instant};

use crate::mpd::playback::PlaybackClientError;

pub struct MpdConnection {
    client: Option<Client>,
    addr: String,
    created_at: Instant,
    max_idle_time: Duration,
    max_age: Duration,
}

impl MpdConnection {
    pub fn new(addr: &str) -> Self {
        Self {
            client: None,
            addr: addr.to_string(),
            created_at: Instant::now(),
            max_idle_time: Duration::from_secs(30),
            max_age: Duration::from_secs(300),
        }
    }

    fn needs_reconnect(&self) -> bool {
        self.client.is_none()
            || self.created_at.elapsed() > self.max_age
            || (self.created_at.elapsed() > self.max_idle_time && self.client.is_some())
    }

    pub fn with_client<F, T>(&mut self, mut f: F) -> Result<T, PlaybackClientError>
    where
        F: FnMut(&mut Client) -> Result<T, PlaybackClientError>,
    {
        if self.needs_reconnect() {
            self.reconnect()?;
        }

        let result = f(self.client.as_mut().unwrap());

        if let Err(PlaybackClientError::MPDConnectionError(_)) = &result {
            self.client = None;
            self.reconnect()?;
            let retry_result = f(self.client.as_mut().unwrap());
            match &retry_result {
                Ok(_) => {
                    self.created_at = Instant::now();
                }
                Err(_) => {
                    self.created_at = Instant::now();
                }
            }
            return retry_result;
        }

        if result.is_ok() {
            self.created_at = Instant::now();
        }

        result
    }

    fn reconnect(&mut self) -> Result<(), PlaybackClientError> {
        let client = Client::connect(&self.addr).map_err(|e| {
            PlaybackClientError::MPDConnectionError(format!("MPD connection failed: {}", e))
        })?;
        self.client = Some(client);
        self.created_at = Instant::now();
        Ok(())
    }
}