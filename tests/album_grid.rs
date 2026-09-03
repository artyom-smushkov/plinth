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

use plinth::mpd::types::{AlbumSortConfig, AlbumSortField};
use plinth::ui::types::Message;
use std::rc::Rc;

#[test]
fn build_groups_with_highest_field() {
    let album1 = make_album("Album A", "Artist X", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album B", "Artist X", vec![make_track("T2", make_song(2, "T2", 200))]);
    let album3 = make_album("Album C", "Artist Y", vec![make_track("T3", make_song(3, "T3", 220))]);

    let albums = Rc::new(vec![album1, album2, album3]);
    let config = AlbumSortConfig {
        highest: Some(AlbumSortField::Artist),
        middle: None,
        lowest: AlbumSortField::Name,
    };
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );
    assert_eq!(grid.group_count(), 2);
    assert_eq!(grid.total_widget_count(), 3);
}

#[test]
fn build_groups_no_highest_single_group() {
    let album1 = make_album("Album A", "Artist X", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album B", "Artist Y", vec![make_track("T2", make_song(2, "T2", 200))]);

    let albums = Rc::new(vec![album1, album2]);
    let config = AlbumSortConfig {
        highest: None,
        middle: None,
        lowest: AlbumSortField::Name,
    };
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );
    assert_eq!(grid.group_count(), 1);
    assert_eq!(grid.total_widget_count(), 2);
}

#[test]
fn grid_thumbnail_ready_updates_widget() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let albums = Rc::new(vec![album]);
    let config = AlbumSortConfig::default();
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let mut grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );

    let result: Result<(String, Vec<u8>), (String, plinth::mpd::playback::PlaybackClientError)> =
        Ok(("grid-0".to_string(), vec![0xFF, 0xD8, 0xFF]));
    let _ = grid.update(Message::GridThumbnailReady("grid-0".to_string(), result));
    assert!(grid.get_widget_thumbnail_loaded(0, 0).unwrap());
}

#[test]
fn grid_album_click_forwards_message() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let albums = Rc::new(vec![album]);
    let config = AlbumSortConfig::default();
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let mut grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );

    let task = grid.update(Message::GridAlbumClicked(0));
    let messages = extract_task_messages(task);
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::GridAlbumClicked(0)));
}

#[test]
fn rebuild_noop_same_config() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let albums = Rc::new(vec![album]);
    let config = AlbumSortConfig::default();
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let mut grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );

    let task = grid.rebuild(&config, 450);
    let messages = extract_task_messages(task);
    assert!(messages.is_empty());
}
#[test]
fn rebuild_with_changed_size_reinitializes_widgets() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let albums = Rc::new(vec![album]);
    let config = AlbumSortConfig::default();
    let albums_order = plinth::mpd::library::sort_albums(albums.clone(), &config);

    let mut grid = plinth::ui::album_grid::AlbumGrid::new(
        albums.clone(),
        Rc::new(albums_order),
        &config,
        450,
    );

    let task = grid.rebuild(&config, 800);
    let messages = extract_task_messages(task);

    assert_eq!(messages.len(), grid.total_widget_count());
    assert!(messages
        .iter()
        .all(|m| matches!(m, Message::GridThumbnailReady(_, _))));
}
