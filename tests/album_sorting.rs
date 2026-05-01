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

use std::rc::Rc;

use plinth::mpd::library::sort_albums;
use plinth::mpd::types::{Album, AlbumSortConfig, AlbumSortField};

fn make_album(genre: &str, artist: &str, date: Option<&str>) -> Rc<Album> {
    Rc::new(Album {
        name: "Test Album".to_string(),
        artist: artist.to_string(),
        genre: genre.to_string(),
        date: date.map(|s| s.to_string()),
        tracks: vec![],
    })
}

#[test]
fn default_sort_genre_artist_date() {
    let albums: Vec<Rc<Album>> = vec![
        make_album("Rock", "Artist B", Some("2023")),
        make_album("Jazz", "Artist A", Some("2024")),
        make_album("Rock", "Artist A", Some("2024")),
        make_album("Rock", "Artist A", Some("2022")),
    ];
    let config = AlbumSortConfig::default();
    let order = sort_albums(Rc::new(albums), &config);

    assert_eq!(order, vec![1, 3, 2, 0]);
}

#[test]
fn sort_only_lowest_field() {
    let albums: Vec<Rc<Album>> = vec![
        make_album("Rock", "Artist A", Some("2023")),
        make_album("Rock", "Artist A", Some("2021")),
        make_album("Rock", "Artist A", Some("2024")),
    ];
    let config = AlbumSortConfig {
        highest: None,
        middle: None,
        lowest: AlbumSortField::Date,
    };
    let order = sort_albums(Rc::new(albums), &config);

    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_empty_albums() {
    let albums: Vec<Rc<Album>> = vec![];
    let config = AlbumSortConfig::default();
    let order = sort_albums(Rc::new(albums), &config);

    assert!(order.is_empty());
}

#[test]
fn sort_key_case_insensitive() {
    let album = make_album("ROCK", "ARTIST", None);
    assert_eq!(album.sort_key(AlbumSortField::Genre), "rock");
    assert_eq!(album.sort_key(AlbumSortField::Artist), "artist");
}

#[test]
fn sort_key_date_none_returns_empty() {
    let album = make_album("Rock", "Artist", None);
    assert_eq!(album.sort_key(AlbumSortField::Date), "");
}

#[test]
fn sort_config_levels_iteration() {
    let config = AlbumSortConfig::default();
    let levels: Vec<_> = config.levels().collect();
    assert_eq!(levels, vec![AlbumSortField::Genre, AlbumSortField::Artist, AlbumSortField::Date]);
}

#[test]
fn sort_stable_for_equal_keys() {
    let albums: Vec<Rc<Album>> = vec![
        make_album("Rock", "Artist A", Some("2023")),
        make_album("Rock", "Artist A", Some("2023")),
    ];
    let config = AlbumSortConfig::default();
    let order = sort_albums(Rc::new(albums), &config);

    assert_eq!(order, vec![0, 1]);
}

#[test]
fn sort_all_fields_none() {
    let albums: Vec<Rc<Album>> = vec![
        make_album("Rock", "Artist B", Some("2023")),
        make_album("Jazz", "Artist A", Some("2024")),
    ];
    let config = AlbumSortConfig {
        highest: None,
        middle: None,
        lowest: AlbumSortField::Name,
    };
    let order = sort_albums(Rc::new(albums), &config);
    assert_eq!(order, vec![0, 1]);
}

#[test]
fn sort_only_highest_set() {
    let albums: Vec<Rc<Album>> = vec![
        make_album("Rock", "Artist B", Some("2023")),
        make_album("Jazz", "Artist A", Some("2024")),
        make_album("Rock", "Artist C", Some("2022")),
    ];
    let config = AlbumSortConfig {
        highest: Some(AlbumSortField::Genre),
        middle: None,
        lowest: AlbumSortField::Artist,
    };
    let order = sort_albums(Rc::new(albums.clone()), &config);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_key_genre_article_handling() {
    let album = Rc::new(Album {
        name: "Test".to_string(),
        artist: "Artist".to_string(),
        genre: "The Rock".to_string(),
        date: None,
        tracks: vec![],
    });
    let key = album.sort_key(AlbumSortField::Genre);
    assert_eq!(key, "rock the");
}

#[test]
fn group_label_all_fields() {
    let album = Rc::new(Album {
        name: "My Album".to_string(),
        artist: "My Artist".to_string(),
        genre: "Rock".to_string(),
        date: Some("2024".to_string()),
        tracks: vec![],
    });
    assert_eq!(album.group_label(AlbumSortField::Name), "My Album");
    assert_eq!(album.group_label(AlbumSortField::Artist), "My Artist");
    assert_eq!(album.group_label(AlbumSortField::Genre), "Rock");
    assert_eq!(album.group_label(AlbumSortField::Date), "2024");
}

#[test]
fn group_label_date_none_returns_unknown() {
    let album = Rc::new(Album {
        name: "My Album".to_string(),
        artist: "My Artist".to_string(),
        genre: "Rock".to_string(),
        date: None,
        tracks: vec![],
    });
    assert_eq!(album.group_label(AlbumSortField::Date), "Unknown");
}

#[test]
fn sort_config_levels_minimal() {
    let config = AlbumSortConfig {
        highest: None,
        middle: None,
        lowest: AlbumSortField::Name,
    };
    let levels: Vec<_> = config.levels().collect();
    assert_eq!(levels, vec![AlbumSortField::Name]);
}

#[test]
fn sort_config_levels_full() {
    let config = AlbumSortConfig {
        highest: Some(AlbumSortField::Genre),
        middle: Some(AlbumSortField::Artist),
        lowest: AlbumSortField::Date,
    };
    let levels: Vec<_> = config.levels().collect();
    assert_eq!(
        levels,
        vec![
            AlbumSortField::Genre,
            AlbumSortField::Artist,
            AlbumSortField::Date
        ]
    );
}