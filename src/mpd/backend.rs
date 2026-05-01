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

use mpd::{Song, Status};

use crate::mpd::playback::PlaybackClientError;

pub trait MpdBackend {
    fn clear(&mut self) -> Result<(), PlaybackClientError>;
    fn push(&mut self, song: Song) -> Result<(), PlaybackClientError>;
    fn delete_range(&mut self, start: u32, end: u32) -> Result<(), PlaybackClientError>;
    fn switch(&mut self, pos: u32) -> Result<(), PlaybackClientError>;
    fn toggle_pause(&mut self) -> Result<(), PlaybackClientError>;
    fn status(&mut self) -> Result<Status, PlaybackClientError>;
    fn seek(&mut self, song_pos: u32, seconds: f64) -> Result<(), PlaybackClientError>;
    fn prev(&mut self) -> Result<(), PlaybackClientError>;
    fn next(&mut self) -> Result<(), PlaybackClientError>;
    fn queue(&mut self) -> Result<Vec<Song>, PlaybackClientError>;
    fn reload_database(&mut self) -> Result<u32, PlaybackClientError>;
}

pub struct RealBackend {
    connection: crate::mpd::connection::MpdConnection,
}

impl RealBackend {
    pub fn new(addr: &str) -> Self {
        Self {
            connection: crate::mpd::connection::MpdConnection::new(addr),
        }
    }
}

impl MpdBackend for RealBackend {
    fn clear(&mut self) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            client.clear().map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to clear queue: {}", e))
            })
        })
    }

    fn push(&mut self, song: Song) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            let _ = client.push(song.clone()).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to push song: {}", e))
            })?;
            Ok(())
        })
    }

    fn delete_range(&mut self, start: u32, end: u32) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            client.delete(start..end).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to delete range {}..{}: {}", start, end, e))
            })
        })
    }

    fn switch(&mut self, pos: u32) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            client.switch(pos).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to switch: {}", e))
            })
        })
    }

    fn toggle_pause(&mut self) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            let _ = client.toggle_pause();
            Ok(())
        })
    }

    fn status(&mut self) -> Result<Status, PlaybackClientError> {
        self.connection.with_client(|client| {
            client.status().map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to get status: {}", e))
            })
        })
    }

    fn seek(&mut self, song_pos: u32, seconds: f64) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            client.seek(song_pos, seconds).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to seek: {}", e))
            })
        })
    }

    fn prev(&mut self) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            let _ = client.prev();
            Ok(())
        })
    }

    fn next(&mut self) -> Result<(), PlaybackClientError> {
        self.connection.with_client(|client| {
            let _ = client.next();
            Ok(())
        })
    }

    fn queue(&mut self) -> Result<Vec<Song>, PlaybackClientError> {
        self.connection.with_client(|client| {
            client.queue().map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to get queue: {}", e))
            })
        })
    }

    fn reload_database(&mut self) -> Result<u32, PlaybackClientError> {
        self.connection.with_client(|client| {
            client.update().map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!("Failed to reload database: {}", e))
            })
        })
    }
}

pub mod test {
    use super::*;
    use mpd::song::{Id, QueuePlace};
    use mpd::status::State;
    use std::time::Duration;

    pub struct MockBackend {
        pub queue: Vec<Song>,
        pub current_pos: u32,
        pub state: State,
        pub elapsed: Option<Duration>,
        pub duration: Option<Duration>,
        pub seek_position: Option<f64>,
        pub reload_count: u32,
    }

    impl MockBackend {
        pub fn new() -> Self {
            Self {
                queue: Vec::new(),
                current_pos: 0,
                state: State::Stop,
                elapsed: None,
                duration: None,
                seek_position: None,
                reload_count: 0,
            }
        }

        pub fn with_state(state: State) -> Self {
            Self {
                state,
                ..Default::default()
            }
        }

        pub fn with_elapsed_duration(elapsed: f64, duration: f64) -> Self {
            Self {
                elapsed: Some(Duration::from_secs_f64(elapsed)),
                duration: Some(Duration::from_secs_f64(duration)),
                ..Default::default()
            }
        }
    }

    impl Default for MockBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MpdBackend for MockBackend {
        fn clear(&mut self) -> Result<(), PlaybackClientError> {
            self.queue.clear();
            Ok(())
        }

        fn push(&mut self, song: Song) -> Result<(), PlaybackClientError> {
            self.queue.push(song);
            Ok(())
        }

        fn delete_range(&mut self, start: u32, end: u32) -> Result<(), PlaybackClientError> {
            let s = start as usize;
            let e = (end as usize).min(self.queue.len());
            if s < self.queue.len() {
                self.queue.drain(s..e);
            }
            if self.current_pos >= self.queue.len() as u32 {
                self.current_pos = self.queue.len().saturating_sub(1) as u32;
            }
            Ok(())
        }

        fn switch(&mut self, pos: u32) -> Result<(), PlaybackClientError> {
            self.current_pos = pos;
            self.state = State::Play;
            self.elapsed = Some(Duration::ZERO);
            Ok(())
        }

        fn toggle_pause(&mut self) -> Result<(), PlaybackClientError> {
            self.state = match self.state {
                State::Play => State::Pause,
                State::Pause => State::Play,
                _ => self.state,
            };
            Ok(())
        }

        fn status(&mut self) -> Result<Status, PlaybackClientError> {
            let song = if self.current_pos < self.queue.len() as u32 {
                Some(QueuePlace {
                    id: Id(self.current_pos),
                    pos: self.current_pos,
                    prio: 0,
                })
            } else {
                None
            };
            Ok(Status {
                state: self.state,
                song,
                elapsed: self.elapsed,
                duration: self.duration,
                ..Default::default()
            })
        }

        fn seek(&mut self, _song_pos: u32, seconds: f64) -> Result<(), PlaybackClientError> {
            self.seek_position = Some(seconds);
            self.elapsed = Some(Duration::from_secs_f64(seconds));
            Ok(())
        }

        fn prev(&mut self) -> Result<(), PlaybackClientError> {
            if self.current_pos > 0 {
                self.current_pos -= 1;
            }
            self.elapsed = Some(Duration::ZERO);
            Ok(())
        }

        fn next(&mut self) -> Result<(), PlaybackClientError> {
            if self.current_pos as usize + 1 < self.queue.len() {
                self.current_pos += 1;
            }
            self.elapsed = Some(Duration::ZERO);
            Ok(())
        }

        fn queue(&mut self) -> Result<Vec<Song>, PlaybackClientError> {
            Ok(self.queue.clone())
        }

        fn reload_database(&mut self) -> Result<u32, PlaybackClientError> {
            self.reload_count += 1;
            Ok(self.reload_count)
        }
    }
}