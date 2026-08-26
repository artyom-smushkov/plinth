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

use plinth::config::ColorScheme;
use plinth::mpd::types::AlbumSortField;
use plinth::ui::main_window::View;
use plinth::ui::types::Message;

#[test]
fn settings_button_clicked() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingsButtonClicked);
    let mw = app.main_window().unwrap();
    assert_eq!(mw.current_view(), View::Settings);
}

#[test]
fn control_hover_sets_hover_state() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::ControlHovered(plinth::ui::types::Control::PlayPause));
    let task = app.update(Message::ControlUnhovered(plinth::ui::types::Control::PlayPause));
    assert!(extract_task_messages(task).is_empty());
}

#[test]
fn sort_highest_change_updates_config() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingSortHighestChanged(Some(AlbumSortField::Artist)));
    assert_eq!(app.settings.sort_config.highest, Some(AlbumSortField::Artist));
}

#[test]
fn sort_middle_change_updates_config() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingSortMiddleChanged(None));
    assert_eq!(app.settings.sort_config.middle, None);
}

#[test]
fn sort_lowest_change_updates_config() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingSortLowestChanged(AlbumSortField::Name));
    assert_eq!(app.settings.sort_config.lowest, AlbumSortField::Name);
}

#[test]
fn colorscheme_change_updates_app() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingColorschemeChanged(ColorScheme::Nord));
    assert_eq!(app.theme(), ColorScheme::Nord.into());
}

#[test]
fn grid_thumbnail_size_change() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingGridThumbnailSizeChanged(600));
    assert_eq!(app.settings.grid_thumbnail_size, 600);
}

#[test]
fn now_playing_thumbnail_size_change() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let _ = app.update(Message::SettingNowPlayingThumbnailSizeChanged(800));
    assert_eq!(app.settings.now_playing_thumbnail_size, 800);
}

#[test]
fn album_grid_rebuild_on_sort_change() {
    let album1 = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let album2 = make_album("Album 2", "Artist B", vec![make_track("T2", make_song(2, "T2", 200))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album1, album2], backend);

    let _ = app.update(Message::SettingSortHighestChanged(Some(AlbumSortField::Artist)));
    assert_eq!(app.settings.sort_config.highest, Some(AlbumSortField::Artist));
}

#[test]
fn now_playing_size_change_reloads_existing_queue_widgets() {
    let album = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = std::cell::RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let target = app.settings.now_playing_thumbnail_size.saturating_add(20);
    let task = app.update(Message::SettingNowPlayingThumbnailSizeChanged(target));
    let messages = extract_task_messages(task);

    assert!(
        messages.iter().any(|m| matches!(m, Message::NowPlayingThumbnailReady(_, _))),
        "changing the now playing size must reload thumbnails of already queued albums"
    );
}

#[test]
fn now_playing_size_change_same_size_does_not_reload() {
    let album = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = std::cell::RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));

    let current = app.settings.now_playing_thumbnail_size;
    let task = app.update(Message::SettingNowPlayingThumbnailSizeChanged(current));
    let messages = extract_task_messages(task);

    assert!(
        messages.iter().all(|m| !matches!(m, Message::NowPlayingThumbnailReady(_, _))),
        "changing to the current size must not reload thumbnails"
    );
}
