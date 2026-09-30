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

use mpd::song::Song;
use plinth::mpd::library::{
    build_albums_from_tracks, get_album_artist, get_tag_value, parse_tag_number,
};
use plinth::mpd::types::Track;
use std::time::Duration;

fn make_song_with_tags(tags: Vec<(&str, &str)>) -> Song {
    Song {
        file: "/music/test.mp3".to_string(),
        title: Some("Test Track".to_string()),
        artist: Some("Artist A".to_string()),
        duration: Some(Duration::from_secs(180)),
        tags: tags
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}

fn make_track_with_song(song: Song) -> Track {
    Track {
        title: "Test Track".to_string(),
        artist: song.artist.clone().unwrap_or_default(),
        cd_number: None,
        track_number: None,
        mpd_song: song,
    }
}

#[test]
fn parse_tag_number_plain() {
    let song = make_song_with_tags(vec![("Track", "3")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), Some(3));
}

#[test]
fn parse_tag_number_fraction() {
    let song = make_song_with_tags(vec![("Track", "3/12")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), Some(3));
}

#[test]
fn parse_tag_number_with_spaces() {
    let song = make_song_with_tags(vec![("Track", " 4 ")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), Some(4));
}

#[test]
fn parse_tag_number_non_numeric() {
    let song = make_song_with_tags(vec![("Track", "abc")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), None);
}

#[test]
fn parse_tag_number_missing_tag() {
    let song = make_song_with_tags(vec![]);
    assert_eq!(parse_tag_number(&song, &["Track"]), None);
}

#[test]
fn parse_tag_number_fallback_keys() {
    let song = make_song_with_tags(vec![("DiscNumber", "2")]);
    assert_eq!(parse_tag_number(&song, &["Disc", "DiscNumber"]), Some(2));
}

#[test]
fn get_tag_value_existing() {
    let song = make_song_with_tags(vec![("Genre", "Rock")]);
    assert_eq!(get_tag_value(&song, "Genre"), Some("Rock".to_string()));
}

#[test]
fn get_tag_value_missing() {
    let song = make_song_with_tags(vec![("Genre", "Rock")]);
    assert_eq!(get_tag_value(&song, "Album"), None);
}

#[test]
fn get_album_artist_from_tag() {
    let song = make_song_with_tags(vec![("Album Artist", "Band X")]);
    let track = make_track_with_song(song);
    assert_eq!(get_album_artist(&track), "Band X");
}

#[test]
fn get_album_artist_fallback_first() {
    let song = make_song_with_tags(vec![]);
    let mut track = make_track_with_song(song);
    track.artist = "Artist A; Artist B".to_string();
    assert_eq!(get_album_artist(&track), "Artist A");
}

#[test]
fn get_album_artist_unknown() {
    let song = Song {
        file: "/music/test.mp3".to_string(),
        title: Some("Test".to_string()),
        artist: None,
        duration: None,
        tags: vec![],
        ..Default::default()
    };
    let track = Track {
        title: "Test".to_string(),
        artist: "".to_string(),
        cd_number: None,
        track_number: None,
        mpd_song: song,
    };
    assert_eq!(get_album_artist(&track), "");
}

#[test]
fn build_albums_groups_by_artist_album() {
    let song1 = make_song_with_tags(vec![("Album Artist", "Artist A")]);
    let song2 = make_song_with_tags(vec![("Album Artist", "Artist B")]);

    let tracks = vec![
        Track {
            title: "Track 1".to_string(),
            artist: "Artist A".to_string(),
            cd_number: None,
            track_number: Some(1),
            mpd_song: song1,
        },
        Track {
            title: "Track 2".to_string(),
            artist: "Artist B".to_string(),
            cd_number: None,
            track_number: Some(2),
            mpd_song: song2,
        },
    ];

    let albums = build_albums_from_tracks("Same Album".to_string(), tracks);
    assert_eq!(albums.len(), 2);
}

#[test]
fn build_albums_single_album() {
    let song1 = make_song_with_tags(vec![("Album Artist", "Artist A")]);
    let song2 = make_song_with_tags(vec![("Album Artist", "Artist A")]);

    let tracks = vec![
        Track {
            title: "Track 1".to_string(),
            artist: "Artist A".to_string(),
            cd_number: None,
            track_number: Some(1),
            mpd_song: song1,
        },
        Track {
            title: "Track 2".to_string(),
            artist: "Artist A".to_string(),
            cd_number: None,
            track_number: Some(2),
            mpd_song: song2,
        },
    ];

    let albums = build_albums_from_tracks("Test Album".to_string(), tracks);
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].tracks.len(), 2);
}

#[test]
fn parse_tag_number_disc_number_fallback() {
    let song = make_song_with_tags(vec![("Disc", "2/4")]);
    assert_eq!(parse_tag_number(&song, &["Disc", "DiscNumber"]), Some(2));
}

#[test]
fn build_albums_genre_and_date_extraction() {
    let song = make_song_with_tags(vec![
        ("Album Artist", "Artist A"),
        ("Genre", "Jazz"),
        ("Date", "2020"),
    ]);

    let tracks = vec![Track {
        title: "Track 1".to_string(),
        artist: "Artist A".to_string(),
        cd_number: None,
        track_number: Some(1),
        mpd_song: song,
    }];

    let albums = build_albums_from_tracks("Test Album".to_string(), tracks);
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].genre, "Jazz");
    assert_eq!(albums[0].date, Some("2020".to_string()));
}

#[test]
fn parse_tag_number_leading_zeros() {
    let song = make_song_with_tags(vec![("Track", "03")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), Some(3));
}

#[test]
fn parse_tag_number_large_number() {
    let song = make_song_with_tags(vec![("Track", "999")]);
    assert_eq!(parse_tag_number(&song, &["Track"]), Some(999));
}
