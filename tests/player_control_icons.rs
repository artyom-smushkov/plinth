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

use plinth::config::AppSettings;
use plinth::ui::main_window::View;
use plinth::ui::player_control::{Icon, PlayerControl};
use plinth::ui::types::{Control, Message};
use std::cell::RefCell;
use iced::widget::canvas::Path;

fn path_points(path: &Path) -> Vec<(f32, f32)> {
    let mut points = vec![];
    for event in path.raw().iter() {
        match event {
            lyon_path::PathEvent::Begin { at } => points.push((at.x, at.y)),
            lyon_path::PathEvent::Line { to, .. } => points.push((to.x, to.y)),
            lyon_path::PathEvent::Quadratic { to, .. } => points.push((to.x, to.y)),
            lyon_path::PathEvent::Cubic { to, .. } => points.push((to.x, to.y)),
            lyon_path::PathEvent::End { .. } => {}
        }
    }
    points
}

#[test]
fn every_icon_fits_design_box() {
    for icon in Icon::all() {
        for path in icon.paths() {
            let points = path_points(&path);
            assert!(!points.is_empty(), "{icon:?} has an empty path");
            for (x, y) in points {
                assert!(
                    (0.0..=24.0).contains(&x),
                    "{icon:?} point ({x}, {y}) escapes the design box"
                );
                assert!(
                    (0.0..=24.0).contains(&y),
                    "{icon:?} point ({x}, {y}) escapes the design box"
                );
            }
        }
    }
}


#[test]
fn play_pause_icon_follows_playback_state() {
    let control = PlayerControl::new();
    assert_eq!(
        control.icon_for(Control::PlayPause, mpd::State::Stop),
        Icon::Play
    );
    assert_eq!(
        control.icon_for(Control::PlayPause, mpd::State::Pause),
        Icon::Play
    );
    assert_eq!(
        control.icon_for(Control::PlayPause, mpd::State::Play),
        Icon::Pause
    );
}

#[test]
fn all_controls_map_to_distinct_icons() {
    let control = PlayerControl::new();
    let icons = [
        Control::AlbumGrid,
        Control::NowPlaying,
        Control::Settings,
        Control::PreviousSong,
        Control::PlayPause,
        Control::NextSong,
        Control::ClearQueue,
        Control::ReloadDatabase,
    ]
    .iter()
    .map(|c| control.icon_for(*c, mpd::State::Stop))
    .collect::<Vec<_>>();

    for i in 0..icons.len() {
        for j in (i + 1)..icons.len() {
            assert_ne!(
                icons[i], icons[j],
                "controls at {i} and {j} share the same icon"
            );
        }
    }
}

#[test]
fn skip_icons_are_horizontal_mirrors() {
    let prev_paths = Icon::Previous.paths();
    let next_paths = Icon::Next.paths();
    assert_eq!(prev_paths.len(), next_paths.len());

    for (prev, next) in prev_paths.iter().zip(next_paths.iter()) {
        let mirrored: Vec<(f32, f32)> = path_points(prev)
            .into_iter()
            .map(|(x, y)| (24.0 - x, y))
            .collect();
        let mut a = mirrored;
        let mut b = path_points(next);
        a.sort_by(|p, q| p.partial_cmp(q).unwrap());
        b.sort_by(|p, q| p.partial_cmp(q).unwrap());
        assert_eq!(a.len(), b.len());
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert!(
                (pa.0 - pb.0).abs() < 1e-3 && (pa.1 - pb.1).abs() < 1e-3,
                "mirror mismatch: {pa:?} vs {pb:?}"
            );
        }
    }
}

#[test]
fn control_hover_state_tracks_messages() {
    let mut control = PlayerControl::new();
    assert_eq!(control.hovered_control(), None);

    let _ = control.update(Message::ControlHovered(Control::PlayPause));
    assert_eq!(control.hovered_control(), Some(Control::PlayPause));

    let _ = control.update(Message::ControlUnhovered(Control::PlayPause));
    assert_eq!(control.hovered_control(), None);
}


#[test]
fn app_view_builds_with_queued_album_in_now_playing() {
    let album = make_album("Album", "Artist", vec![make_track("T1", make_song(1, "T1", 180))]);
    let backend = MockBackend::new();
    let mut app = make_app(vec![album], backend);

    let queue = RefCell::new(vec![0]);
    let _ = app.update(Message::SyncQueue(queue));
    let _ = app.update(Message::NowPlayingButtonClicked);

    let _ = app.view();
}
