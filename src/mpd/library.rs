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

use crate::mpd::connection::MpdConnection;
use crate::mpd::playback::PlaybackClientError;

use super::types::{Album, AlbumSortConfig, Track};
use dirs::cache_dir;
use image::imageops::FilterType;
use image::ImageReader;
use itertools::Itertools;
use mpd::song::Song;
use mpd::{Client, Query, Term};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use std::rc::Rc;

pub fn load_albums() -> Result<Rc<Vec<Rc<Album>>>, PlaybackClientError> {
    let mut connection = MpdConnection::new("127.0.0.1:6600");
    connection.with_client(|mpd_client| {
        let album_names = mpd_client
            .list(&Term::Tag(Cow::Borrowed("Album")), &Query::default())
            .map_err(|e| {
                PlaybackClientError::MPDRetrieveError(format!(
                    "failed to list albums from MPD: {}",
                    e
                ))
            })?;

        let mut albums = Vec::new();
        for album_name in album_names {
            let tracks = fetch_tracks(mpd_client, album_name.clone())?;
            albums.extend(build_albums_from_tracks(album_name, tracks));
        }
        Ok(Rc::new(albums))
    })
}

fn fetch_tracks(
    mpd_client: &mut Client,
    album_name: String,
) -> Result<Vec<Track>, PlaybackClientError> {
    let mut album_query = Query::default();
    album_query.and(Term::Tag(Cow::Borrowed("Album")), album_name.as_str());

    let tracks: Vec<_> = mpd_client
        .find(&album_query, None)
        .map_err(|e| {
            PlaybackClientError::MPDRetrieveError(format!(
                "failed to search tracks for album '{}': {}",
                album_name, e
            ))
        })?
        .into_iter()
        .map(|song| Track {
            title: song.title.clone().unwrap_or_else(|| song.file.clone()),
            artist: song.artist.clone().unwrap_or_default(),
            cd_number: parse_tag_number(&song, &["Disk", "DiscNumber"]),
            track_number: parse_tag_number(&song, &["Track"]),
            mpd_song: song,
        })
        .sorted_by_key(|track| {
            format!(
                "{:03}{:03}",
                track.cd_number.unwrap_or(0),
                track.track_number.unwrap_or(0)
            )
        })
        .collect();

    Ok(tracks)
}

pub fn build_albums_from_tracks(album_name: String, tracks: Vec<Track>) -> Vec<Rc<Album>> {
    let mut albums_map: HashMap<String, Album> = HashMap::new();

    for track in tracks {
        let album_artist = get_album_artist(&track);
        let key = format!("{}||{}", album_artist, album_name);
        match albums_map.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().tracks.push(track);
            }
            Entry::Vacant(entry) => {
                let genre = get_tag_value(&track.mpd_song, "Genre").unwrap_or_default();
                let date = get_tag_value(&track.mpd_song, "Date");

                entry.insert(Album {
                    name: album_name.clone(),
                    artist: album_artist,
                    genre,
                    date,
                    tracks: vec![track],
                });
            }
        }
    }
    albums_map.into_values().map(Rc::new).collect()
}

pub fn get_album_artist(track: &Track) -> String {
    get_tag_value(&track.mpd_song, "Album Artist")
        .or_else(|| {
            let artist = &track.artist;
            artist.split(';').next().map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown artist".to_string())
}

pub fn get_album_key_from_song(song: &Song) -> (String, String) {
    let album_name = get_tag_value(song, "Album").unwrap_or_default();
    let album_artist = get_tag_value(song, "Album Artist")
        .or_else(|| {
            song.artist.as_ref().and_then(|a| {
                let trimmed = a.split(';').next()?.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            })
        })
        .unwrap_or_else(|| "Unknown artist".to_string());
    (album_artist, album_name)
}

pub fn sort_albums(
    albums: Rc<Vec<Rc<Album>>>,
    config: &AlbumSortConfig,
) -> Vec<usize> {
    let mut indices = (0..albums.len()).collect::<Vec<_>>();
    indices.sort_by(|&i, &j| {
        config
            .levels()
            .map(|field| albums[i].sort_key(field).cmp(&albums[j].sort_key(field)))
            .find(|&ord| ord != std::cmp::Ordering::Equal)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
}

pub fn get_tag_value(song: &Song, key: &str) -> Option<String> {
    song.tags.iter().find_map(|(tag, value)| {
        if tag == key {
            Some(value.clone())
        } else {
            None
        }
    })
}

pub fn parse_tag_number(song: &Song, keys: &[&str]) -> Option<u32> {
    keys.iter().find_map(|key| {
        get_tag_value(song, key).and_then(|value| {
            value
                .split('/')
                .next()
                .map(str::trim)
                .and_then(|part| part.parse::<u32>().ok())
        })
    })
}

fn get_cache_dir() -> PathBuf {
    let cache_dir = cache_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("plinth")
        .join("thumbnails");
    fs::create_dir_all(&cache_dir).ok();
    cache_dir
}

fn hash_data(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

fn get_cover_data(mpd_song: &Song) -> Result<Vec<u8>, PlaybackClientError> {
    let mut connection = MpdConnection::new("127.0.0.1:6600");
    connection.with_client(|mpd_client| {
        mpd_client.albumart(mpd_song).map_err(|e| {
            PlaybackClientError::CoverArtError(format!("failed to get cover art: {}", e))
        })
    })
}

fn create_thumbnail(
    cover_data: &[u8],
    size: u32,
    album_name: &str,
) -> Result<Vec<u8>, PlaybackClientError> {
    let img = ImageReader::new(Cursor::new(cover_data))
        .with_guessed_format()
        .map_err(|e| {
            PlaybackClientError::CoverArtError(format!(
                "failed to guess {} album image format: {}",
                album_name, e
            ))
        })?
        .decode()
        .map_err(|e| {
            PlaybackClientError::CoverArtError(format!(
                "failed to decode {} album image: {}",
                album_name, e
            ))
        })?;

    let thumbnail_img = img.resize(size, size, FilterType::Nearest);

    let mut thumbnail: Vec<u8> = Vec::new();
    thumbnail_img
        .write_to(Cursor::new(&mut thumbnail), image::ImageFormat::Jpeg)
        .map_err(|e| {
            PlaybackClientError::CoverArtError(format!(
                "failed to encode {} album image: {}",
                album_name, e
            ))
        })?;

    Ok(thumbnail)
}

fn cleanup_old_thumbnails(cache_dir: &PathBuf, old_pattern: &str, current_path: &PathBuf) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if file_name_str.starts_with(old_pattern) && &PathBuf::from(&file_name) != current_path {
            fs::remove_file(entry.path()).ok();
        }
    }
}

pub fn get_thumbnail(
    first_song: &Song,
    album_artist: &str,
    album_name: &str,
    size: u32,
) -> Result<Vec<u8>, PlaybackClientError> {
    let cover_data = get_cover_data(first_song)?;
    let cover_hash = hash_data(&cover_data);

    let cache_dir = get_cache_dir();
    let cache_key = format!(
        "{}_{}_{}_{}_{}",
        album_artist.replace('/', "-"),
        album_name.replace('/', "-"),
        size,
        cover_hash,
        "jpg"
    );
    let cache_path = cache_dir.join(&cache_key);

    if let Ok(cached) = fs::read(&cache_path) {
        return Ok(cached);
    }

    let thumbnail = create_thumbnail(&cover_data, size, album_name)?;

    let old_pattern = format!(
        "{}_{}_{}",
        album_artist.replace('/', "-"),
        album_name.replace('/', "-"),
        size
    );
    cleanup_old_thumbnails(&cache_dir, &old_pattern, &cache_path);

    fs::write(&cache_path, &thumbnail).ok();

    Ok(thumbnail)
}