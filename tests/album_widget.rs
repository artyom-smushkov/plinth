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
use plinth::ui::album::{AlbumDisplayOption, AlbumWidget};
use plinth::ui::types::Message;

#[test]
fn album_widget_grid_id_format() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let widget = AlbumWidget::new(album, Some(5), AlbumDisplayOption::Grid, 450);
    assert_eq!(widget.widget_id, "grid-5");
}

#[test]
fn album_widget_now_playing_id_format() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let widget = AlbumWidget::new(album, None, AlbumDisplayOption::NowPlaying, 1000);
    assert_eq!(widget.widget_id, "now-playing-Artist-Album");
}

#[test]
fn album_widget_initial_state_not_loaded() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);
    assert!(!widget.is_thumbnail_loaded());
}

#[test]
fn album_widget_thumbnail_loaded_success() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);

    let result: Result<(String, Vec<u8>), (String, PlaybackClientError)> =
        Ok(("grid-0".to_string(), vec![0xFF, 0xD8, 0xFF]));
    let _ = widget.update(Message::GridThumbnailReady("grid-0".to_string(), result));
    assert!(widget.is_thumbnail_loaded());
}

#[test]
fn album_widget_thumbnail_loaded_error() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);

    let result: Result<(String, Vec<u8>), (String, PlaybackClientError)> = Err((
        "grid-0".to_string(),
        PlaybackClientError::CoverArtError("test error".into()),
    ));
    let _ = widget.update(Message::GridThumbnailReady("grid-0".to_string(), result));
    assert!(!widget.is_thumbnail_loaded());
}

#[test]
fn album_widget_ignores_wrong_widget_id() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);

    let result: Result<(String, Vec<u8>), (String, PlaybackClientError)> =
        Ok(("grid-1".to_string(), vec![0xFF, 0xD8, 0xFF]));
    let _ = widget.update(Message::GridThumbnailReady("grid-1".to_string(), result));
    assert!(!widget.is_thumbnail_loaded());
}

#[test]
fn album_widget_now_playing_thumbnail() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, None, AlbumDisplayOption::NowPlaying, 1000);

    let wid = widget.widget_id.clone();
    let result: Result<(String, Vec<u8>), (String, PlaybackClientError)> =
        Ok((wid.clone(), vec![0xFF, 0xD8, 0xFF]));
    let _ = widget.update(Message::NowPlayingThumbnailReady(wid, result));
    assert!(widget.is_thumbnail_loaded());
}
#[test]
fn set_thumbnail_size_same_size_does_not_reload() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);

    let task = widget.set_thumbnail_size(450);
    assert!(extract_task_messages(task).is_empty());
}

#[test]
fn set_thumbnail_size_new_size_reloads_thumbnail() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, Some(0), AlbumDisplayOption::Grid, 450);

    let task = widget.set_thumbnail_size(800);
    let messages = extract_task_messages(task);

    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::GridThumbnailReady(_, _)));
}

#[test]
fn set_thumbnail_size_now_playing_reloads_at_new_size() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let mut widget = AlbumWidget::new(album, None, AlbumDisplayOption::NowPlaying, 1000);

    let task = widget.set_thumbnail_size(920);
    let messages = extract_task_messages(task);

    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::NowPlayingThumbnailReady(_, _)));
}
