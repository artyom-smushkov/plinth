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

use plinth::ui::types::Message;

fn make_song_no_duration(id: u32, title: &str) -> mpd::song::Song {
    mpd::song::Song {
        file: format!("/music/{}.mp3", id),
        title: Some(title.to_string()),
        artist: Some("Test Artist".to_string()),
        duration: None,
        tags: vec![(String::from("Album"), String::from("Test Album"))],
        ..Default::default()
    }
}

fn make_track_no_duration(title: &str, song: mpd::song::Song) -> plinth::mpd::types::Track {
    plinth::mpd::types::Track {
        title: title.to_string(),
        artist: "Test Artist".to_string(),
        cd_number: None,
        track_number: None,
        mpd_song: song,
    }
}

#[test]
fn track_widget_format_duration() {
    let song = make_song(1, "T1", 245);
    let track = make_track("T1", song);
    let widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);
    assert_eq!(widget.format_duration(), "04:05");
}

#[test]
fn track_widget_format_duration_missing() {
    let song = make_song_no_duration(1, "T1");
    let track = make_track_no_duration("T1", song);
    let widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);
    assert_eq!(widget.format_duration(), "--:--");
}

#[test]
fn track_widget_initial_state() {
    let song = make_song(1, "T1", 180);
    let track = make_track("T1", song);
    let widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);
    assert!(!widget.is_current_track());
    assert!(!widget.is_hovered());
}

#[test]
fn track_widget_hover_unhover() {
    let song = make_song(1, "T1", 180);
    let track = make_track("T1", song);
    let mut widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);

    let _ = widget.update(Message::TrackHovered(0, 0));
    assert!(widget.is_hovered());

    let _ = widget.update(Message::TrackUnhovered(0, 0));
    assert!(!widget.is_hovered());
}

#[test]
fn track_widget_started_ended_playing() {
    let song = make_song(1, "T1", 180);
    let track = make_track("T1", song);
    let mut widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);

    assert!(!widget.is_current_track());

    let _ = widget.update(Message::TrackStartedPlaying(0, 0));
    assert!(widget.is_current_track());

    let _ = widget.update(Message::TrackEndedPlaying(0, 0));
    assert!(!widget.is_current_track());
}

#[test]
fn track_widget_ignores_wrong_indices() {
    let song = make_song(1, "T1", 180);
    let track = make_track("T1", song);
    let mut widget = plinth::ui::track::TrackWidget::new(track, 0, 0, false);

    let _ = widget.update(Message::TrackHovered(1, 0));
    assert!(!widget.is_hovered());

    let _ = widget.update(Message::TrackStartedPlaying(1, 0));
    assert!(!widget.is_current_track());
}

#[test]
fn track_widget_created_as_current() {
    let song = make_song(1, "T1", 180);
    let track = make_track("T1", song);
    let widget = plinth::ui::track::TrackWidget::new(track, 0, 0, true);
    assert!(widget.is_current_track());
}