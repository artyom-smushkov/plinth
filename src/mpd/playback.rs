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

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::backend::{MpdBackend, RealBackend};
use super::library::{load_albums, sort_albums};
use super::types::{Album, AlbumSortConfig, PlaybackCursor};
use mpd::State;

pub struct PlaybackClient {
    pub(crate) albums: Rc<Vec<Rc<Album>>>,
    pub(crate) albums_order: Rc<Vec<usize>>,
    pub(crate) sort_config: AlbumSortConfig,
    pub queue: RefCell<Vec<usize>>,
    pub cursor: Option<Rc<PlaybackCursor>>,
    pub backend: Box<dyn MpdBackend>,
    pub(crate) test_reload_result: Option<Result<Rc<Vec<Rc<Album>>>, PlaybackClientError>>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PlaybackClientError {
    MPDConnectionError(String),
    MPDRetrieveError(String),
    CoverArtError(String),
    InvalidAlbumIndexError,
}

impl PlaybackClient {
    pub fn new() -> Result<PlaybackClient, PlaybackClientError> {
        let backend = Box::new(RealBackend::new("127.0.0.1:6600"));
        let albums = load_albums()?;
        let sort_config = AlbumSortConfig::default();
        let albums_order = sort_albums(albums.clone(), &sort_config);

        Ok(Self {
            albums: albums,
            albums_order: Rc::new(albums_order),
            sort_config,
            queue: RefCell::new(Vec::new()),
            cursor: None,
            backend,
            test_reload_result: None,
        })
    }

    pub fn new_for_test(
        albums: Rc<Vec<Rc<Album>>>,
        backend: Box<dyn MpdBackend>,
    ) -> Self {
        let sort_config = AlbumSortConfig::default();
        let albums_order = sort_albums(albums.clone(), &sort_config);

        Self {
            albums,
            albums_order: Rc::new(albums_order),
            sort_config,
            queue: RefCell::new(Vec::new()),
            cursor: None,
            backend,
            test_reload_result: None,
        }
    }

    pub fn init_queue_from_mpd(&mut self) -> Result<(), PlaybackClientError> {
        let mpd_queue = self.backend.queue()?;
        if mpd_queue.is_empty() {
            return Ok(());
        }

        let file_to_album: HashMap<String, usize> = self.albums.iter().enumerate().flat_map(|(album_idx, album)| {
            album.tracks.iter().map(move |track| (track.mpd_song.file.clone(), album_idx))
        }).collect();

        let mut album_sequence: Vec<usize> = Vec::new();
        let mut seen_albums = HashSet::new();
        let mut album_track_counts: HashMap<usize, usize> = HashMap::new();
        let mut needs_rebuild = false;

        for song in &mpd_queue {
            let album_idx = match file_to_album.get(&song.file) {
                Some(idx) => *idx,
                None => {
                    needs_rebuild = true;
                    break;
                }
            };
            *album_track_counts.entry(album_idx).or_insert(0) += 1;
            if !seen_albums.contains(&album_idx) {
                seen_albums.insert(album_idx);
                album_sequence.push(album_idx);
            } else if let Some(&last) = album_sequence.last()
                && last != album_idx
            {
                needs_rebuild = true;
                break;
            }
        }

        if !needs_rebuild {
            for &album_idx in &album_sequence {
                let expected = self.albums[album_idx].tracks.len();
                let actual = album_track_counts.get(&album_idx).copied().unwrap_or(0);
                if actual != expected {
                    needs_rebuild = true;
                    break;
                }
            }
        }

        if needs_rebuild {
            self.rebuild_queue(&mpd_queue, &file_to_album)?;
            self.restore_cursor_if_playing()?;
            return Ok(());
        }

        self.queue.borrow_mut().extend(album_sequence);
        self.restore_cursor_if_playing()?;
        Ok(())
    }

    fn rebuild_queue(&mut self, mpd_queue: &[mpd::Song], file_to_album: &HashMap<String, usize>) -> Result<(), PlaybackClientError> {
        let mut ordered_albums: Vec<usize> = Vec::new();
        let mut seen = HashSet::new();

        for song in mpd_queue {
            if let Some(&album_idx) = file_to_album.get(&song.file)
                && seen.insert(album_idx)
            {
                ordered_albums.push(album_idx);
            }
        }

        self.backend.clear()?;
        self.queue.borrow_mut().clear();

        for &album_idx in &ordered_albums {
            self.enqueue_tracks(album_idx)?;
            self.queue.borrow_mut().push(album_idx);
        }

        Ok(())
    }

    fn restore_cursor_if_playing(&mut self) -> Result<(), PlaybackClientError> {
        let status = self.backend.status()?;
        let Some(song_pos) = status.song else {
            return Ok(());
        };

        let pos = song_pos.pos as usize;
        let queue = self.queue.borrow();
        let mut cumulative = 0;
        for (queue_idx, &album_idx) in queue.iter().enumerate() {
            let num_tracks = self.albums[album_idx].tracks.len();
            if pos >= cumulative && pos < cumulative + num_tracks {
                let track_idx = pos - cumulative;
                drop(queue);
                self.cursor = Some(Rc::new(PlaybackCursor {
                    queued_album_index: queue_idx,
                    track_index: track_idx,
                    position_ms: 0,
                }));
                return Ok(());
            }
            cumulative += num_tracks;
        }

        Ok(())
    }

    pub fn play_album(&mut self, album_index: usize) -> Result<(), PlaybackClientError> {
        if !self.album_index_is_valid(album_index) {
            return Err(PlaybackClientError::InvalidAlbumIndexError);
        }

        self.queue.borrow_mut().clear();

        let tracks: Vec<_> = self.albums[album_index].tracks.iter().cloned().collect();

        self.backend.clear().map_err(|e| {
            PlaybackClientError::MPDRetrieveError(format!(
                "failed to clear MPD queue before playing album index {album_index}: {e:?}"
            ))
        })?;

        for track in &tracks {
            self.backend.push(track.mpd_song.clone()).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!(
                    "Failed to add track {} - {}: {:?}",
                    self.albums[album_index].name,
                    track.title,
                    e
                ))
            })?;
        }

        self.backend.switch(0).map_err(|e| {
            PlaybackClientError::MPDRetrieveError(format!(
                "failed to start playback for album index {album_index}: {e:?}"
            ))
        })?;

        self.queue.borrow_mut().push(album_index);
        let queued_album_index = self.queue.borrow().len() - 1;
        self.cursor = Some(Rc::new(PlaybackCursor {
            queued_album_index: queued_album_index,
            track_index: 0,
            position_ms: 0,
        }));
        Ok(())
    }

    pub fn add_album_to_queue(&mut self, album_index: usize) -> Result<(), PlaybackClientError> {
        if !self.album_index_is_valid(album_index) {
            return Err(PlaybackClientError::InvalidAlbumIndexError);
        }

        if self.queue.borrow().is_empty() {
            self.play_album(album_index)?;
            return Ok(());
        }

        if self.queue.borrow().contains(&album_index) {
            return Ok(());
        }

        self.enqueue_tracks(album_index)?;
        self.queue.borrow_mut().push(album_index);
        Ok(())
    }

    pub fn switch_to_track(
        &mut self,
        queue_index: usize,
        track_index: usize,
    ) -> Result<(), PlaybackClientError> {
        if queue_index >= self.queue.borrow().len() {
            return Ok(());
        }

        let album_index = self.queue.borrow()[queue_index];
        if track_index >= self.albums[album_index].tracks.len() {
            return Ok(());
        }

        let mpd_queue_position = self.queue.borrow()[..queue_index]
            .iter()
            .fold(0, |acc, i| acc + self.albums[*i].tracks.len())
            + track_index;

        self.backend.switch(mpd_queue_position as u32).map_err(|e| {
            PlaybackClientError::MPDRetrieveError(format!(
                "failed to switch to queue index {queue_index}, track index {track_index} (MPD position {mpd_queue_position}): {e:?}"
            ))
        })?;

        self.cursor = Some(Rc::new(PlaybackCursor {
            queued_album_index: queue_index,
            track_index: track_index,
            position_ms: 0,
        }));
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), PlaybackClientError> {
        self.backend.toggle_pause()
    }

    pub fn clear_queue(&mut self) -> Result<(), PlaybackClientError> {
        self.backend.clear()?;
        self.queue.borrow_mut().clear();
        self.cursor = None;
        Ok(())
    }

    pub fn remove_album_from_queue(&mut self, queue_index: usize) -> Result<(), PlaybackClientError> {
        let queue = self.queue.borrow();
        if queue_index >= queue.len() {
            return Err(PlaybackClientError::InvalidAlbumIndexError);
        }

        let album_index = queue[queue_index];
        let album = &self.albums[album_index];
        let num_tracks = album.tracks.len();

        let mpd_start: u32 = queue[..queue_index]
            .iter()
            .fold(0, |acc, i| acc + self.albums[*i].tracks.len() as u32);
        let mpd_end = mpd_start + num_tracks as u32;

        drop(queue);

        if queue_index == 0 && self.queue.borrow().len() == 1 {
            self.clear_queue()?;
            return Ok(());
        }

        self.backend.delete_range(mpd_start, mpd_end)?;
        self.queue.borrow_mut().remove(queue_index);

        if let Some(cursor) = &self.cursor {
            if cursor.queued_album_index == queue_index {
                self.cursor = None;
            } else if cursor.queued_album_index > queue_index {
                let new_cursor = PlaybackCursor {
                    queued_album_index: cursor.queued_album_index - 1,
                    track_index: cursor.track_index,
                    position_ms: cursor.position_ms,
                };
                self.cursor = Some(Rc::new(new_cursor));
            }
        }

        Ok(())
    }

    pub fn playback_state(&mut self) -> Result<State, PlaybackClientError> {
        self.backend.status().map(|s| s.state)
    }

    pub fn current_song_position(&mut self) -> Result<Option<usize>, PlaybackClientError> {
        self.backend
            .status()
            .map(|s| s.song.map(|s| s.pos as usize))
    }

    pub fn get_full_status(
        &mut self,
    ) -> Result<(State, Option<f64>, Option<f64>, Option<usize>), PlaybackClientError> {
        let status = self.backend.status()?;
        let state = status.state;
        let elapsed = status.elapsed.map(|d| d.as_secs_f64());
        let duration = status.duration.map(|d| d.as_secs_f64());
        let song_pos = status.song.map(|s| s.pos as usize);
        Ok((state, elapsed, duration, song_pos))
    }

    pub fn seek_to(&mut self, seconds: f64) -> Result<(), PlaybackClientError> {
        let status = self.backend.status()?;
        let pos = status.song.map(|s| s.pos).unwrap_or(0);
        self.backend.seek(pos, seconds)
    }

    pub fn get_elapsed_duration(&mut self) -> Result<(Option<f64>, Option<f64>), PlaybackClientError> {
        let status = self.backend.status()?;
        let elapsed = status.elapsed.map(|d| d.as_secs_f64());
        let duration = status.duration.map(|d| d.as_secs_f64());
        Ok((elapsed, duration))
    }

    pub fn previous_song(&mut self) -> Result<(), PlaybackClientError> {
        self.backend.prev()
    }

    pub fn next_song(&mut self) -> Result<(), PlaybackClientError> {
        self.backend.next()
    }

    pub fn toggle_play_pause(&mut self) -> Result<(), PlaybackClientError> {
        self.pause()
    }

    pub fn reload_database(&mut self) -> Result<u32, PlaybackClientError> {
        self.backend.reload_database()
    }

    pub fn reload_albums(&mut self) -> Result<(), PlaybackClientError> {
        let new_albums = match self.test_reload_result.take() {
            Some(result) => result?,
            None => load_albums()?,
        };
        let new_order = sort_albums(new_albums.clone(), &self.sort_config);
        self.albums = new_albums;
        self.albums_order = Rc::new(new_order);
        self.queue.borrow_mut().clear();
        self.cursor = None;
        Ok(())
    }

    pub fn set_test_reload_result(&mut self, result: Result<Rc<Vec<Rc<Album>>>, PlaybackClientError>) {
        self.test_reload_result = Some(result);
    }

    fn album_index_is_valid(&self, album_index: usize) -> bool {
        if album_index >= self.albums.len() {
            return false;
        }

        !self.albums[album_index].tracks.is_empty()
    }

    fn enqueue_tracks(&mut self, album_index: usize) -> Result<(), PlaybackClientError> {
        let tracks: Vec<_> = self.albums[album_index].tracks.iter().cloned().collect();
        let album_name = self.albums[album_index].name.clone();

        for track in &tracks {
            self.backend.push(track.mpd_song.clone()).map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!(
                    "Failed to add track {} - {}: {:?}",
                    album_name, track.title, e
                ))
            })?;
        }
        Ok(())
    }
}