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

use futures::StreamExt;
use iced_runtime::core::widget::operation::scrollable::{
    AbsoluteOffset, RelativeOffset, Scrollable as ScrollableState,
};
use iced_runtime::core::widget::{Id, Operation};
use iced_runtime::core::{Point, Rectangle, Size, Vector};
use plinth::ui::album_grid::AlbumGrid;
use plinth::ui::App;
use plinth::ui::main_window::View;
use plinth::ui::now_playing::NowPlayingWidget;
use plinth::ui::settings::SettingsWidget;
use plinth::ui::types::Message;

struct RecordingScrollable {
    scroll_to: Vec<AbsoluteOffset<Option<f32>>>,
    snap_to: Vec<RelativeOffset<Option<f32>>>,
    scroll_by: Vec<AbsoluteOffset>,
}

impl Default for RecordingScrollable {
    fn default() -> Self {
        Self {
            scroll_to: Vec::new(),
            snap_to: Vec::new(),
            scroll_by: Vec::new(),
        }
    }
}

impl ScrollableState for RecordingScrollable {
    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>) {
        self.snap_to.push(offset);
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        self.scroll_to.push(offset);
    }

    fn scroll_by(
        &mut self,
        offset: AbsoluteOffset,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
    ) {
        self.scroll_by.push(offset);
    }
}

fn widget_operations(app: &mut App, message: Message) -> Vec<Box<dyn Operation>> {
    let task = app.update(message);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut operations = Vec::new();
            if let Some(mut stream) = iced_runtime::task::into_stream(task) {
                while let Some(action) = stream.next().await {
                    if let iced_runtime::Action::Widget(operation) = action {
                        operations.push(operation);
                    }
                }
            }
            operations
        })
}

fn apply_operations(operations: &mut [Box<dyn Operation>], id: &Id) -> RecordingScrollable {
    let mut mock = RecordingScrollable::default();
    for operation in operations.iter_mut() {
        operation.scrollable(
            Some(id),
            Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            Rectangle::new(Point::ORIGIN, Size::new(800.0, 5000.0)),
            Vector::ZERO,
            &mut mock,
        );
    }
    mock
}

#[test]
fn view_scrollable_ids_are_distinct() {
    assert_ne!(AlbumGrid::SCROLLABLE_ID, NowPlayingWidget::SCROLLABLE_ID);
    assert_ne!(AlbumGrid::SCROLLABLE_ID, SettingsWidget::SCROLLABLE_ID);
    assert_ne!(NowPlayingWidget::SCROLLABLE_ID, SettingsWidget::SCROLLABLE_ID);
}

#[test]
fn each_view_remembers_its_own_scroll_offset() {
    let albums = vec![];
    let backend = MockBackend::new();
    let mut app = make_app(albums, backend);

    let grid_offset = AbsoluteOffset { x: 0.0, y: 100.0 };
    let now_playing_offset = AbsoluteOffset { x: 0.0, y: 200.0 };
    let settings_offset = AbsoluteOffset { x: 0.0, y: 300.0 };

    let _ = app.update(Message::ViewScrolled(View::AlbumGrid, grid_offset));
    let _ = app.update(Message::ViewScrolled(View::NowPlaying, now_playing_offset));
    let _ = app.update(Message::ViewScrolled(View::Settings, settings_offset));

    let main_window = app.main_window().unwrap();
    assert_eq!(main_window.view_scroll(View::AlbumGrid), grid_offset);
    assert_eq!(main_window.view_scroll(View::NowPlaying), now_playing_offset);
    assert_eq!(main_window.view_scroll(View::Settings), settings_offset);
}

#[test]
fn switching_to_now_playing_restores_now_playing_offset_not_grid_offset() {
    let album = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ViewScrolled(
        View::AlbumGrid,
        AbsoluteOffset { x: 0.0, y: 1234.0 },
    ));
    let _ = app.update(Message::ViewScrolled(
        View::NowPlaying,
        AbsoluteOffset { x: 0.0, y: 500.0 },
    ));

    let mut operations = widget_operations(&mut app, Message::NowPlayingButtonClicked);
    assert_eq!(app.main_window().unwrap().current_view(), View::NowPlaying);
    assert!(!operations.is_empty(), "view switch must issue a scroll operation");

    let now_playing = apply_operations(&mut operations, &NowPlayingWidget::SCROLLABLE_ID);
    assert_eq!(
        now_playing.scroll_to,
        vec![AbsoluteOffset { x: Some(0.0), y: Some(500.0) }],
        "now playing view must be restored to its own last offset, not the grid's"
    );

    let grid = apply_operations(&mut operations, &AlbumGrid::SCROLLABLE_ID);
    assert!(
        grid.scroll_to.is_empty() && grid.snap_to.is_empty() && grid.scroll_by.is_empty(),
        "switching to now playing must not scroll the album grid"
    );
}

#[test]
fn switching_back_to_album_grid_restores_grid_offset() {
    let album = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ViewScrolled(
        View::AlbumGrid,
        AbsoluteOffset { x: 0.0, y: 1234.0 },
    ));

    let _ = widget_operations(&mut app, Message::NowPlayingButtonClicked);
    let _ = app.update(Message::ViewScrolled(
        View::NowPlaying,
        AbsoluteOffset { x: 0.0, y: 500.0 },
    ));

    let mut operations = widget_operations(&mut app, Message::AlbumGridButtonClicked);
    assert_eq!(app.main_window().unwrap().current_view(), View::AlbumGrid);

    let grid = apply_operations(&mut operations, &AlbumGrid::SCROLLABLE_ID);
    assert_eq!(
        grid.scroll_to,
        vec![AbsoluteOffset { x: Some(0.0), y: Some(1234.0) }],
        "album grid must be restored to its own last offset after scrolling now playing"
    );

    let now_playing = apply_operations(&mut operations, &NowPlayingWidget::SCROLLABLE_ID);
    assert!(
        now_playing.scroll_to.is_empty() && now_playing.snap_to.is_empty(),
        "switching to the album grid must not scroll now playing"
    );
}

#[test]
fn unscrolled_view_starts_at_top_after_switch() {
    let album = make_album("Album 1", "Artist A", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let _ = app.update(Message::ViewScrolled(
        View::AlbumGrid,
        AbsoluteOffset { x: 0.0, y: 5000.0 },
    ));

    let mut operations = widget_operations(&mut app, Message::NowPlayingButtonClicked);

    let now_playing = apply_operations(&mut operations, &NowPlayingWidget::SCROLLABLE_ID);
    assert_eq!(
        now_playing.scroll_to,
        vec![AbsoluteOffset { x: Some(0.0), y: Some(0.0) }],
        "a view that was never scrolled must start at the top, not inherit the grid's offset"
    );
}
