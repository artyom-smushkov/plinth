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

use plinth::config::AppSettings;
use plinth::ui::settings::{
    GRID_THUMBNAIL_MAX, GRID_THUMBNAIL_MIN, NOW_PLAYING_THUMBNAIL_MAX, NOW_PLAYING_THUMBNAIL_MIN,
    SettingsWidget, THUMBNAIL_STEP,
};

fn settings_with(grid: u32, now_playing: u32) -> AppSettings {
    AppSettings {
        grid_thumbnail_size: grid,
        now_playing_thumbnail_size: now_playing,
        ..AppSettings::default()
    }
}

#[test]
fn grid_size_below_range_is_clamped_to_minimum() {
    let widget = SettingsWidget::new(settings_with(0, 450));
    assert_eq!(widget.clamped_grid_thumbnail_size(), GRID_THUMBNAIL_MIN);

    let widget = SettingsWidget::new(settings_with(GRID_THUMBNAIL_MIN - 1, 450));
    assert_eq!(widget.clamped_grid_thumbnail_size(), GRID_THUMBNAIL_MIN);
}

#[test]
fn grid_size_above_range_is_clamped_to_maximum() {
    let widget = SettingsWidget::new(settings_with(GRID_THUMBNAIL_MAX + 1, 450));
    assert_eq!(widget.clamped_grid_thumbnail_size(), GRID_THUMBNAIL_MAX);

    let widget = SettingsWidget::new(settings_with(u32::MAX, 450));
    assert_eq!(widget.clamped_grid_thumbnail_size(), GRID_THUMBNAIL_MAX);
}

#[test]
fn now_playing_size_is_clamped_to_its_range() {
    let widget = SettingsWidget::new(settings_with(450, 0));
    assert_eq!(widget.clamped_now_playing_thumbnail_size(), NOW_PLAYING_THUMBNAIL_MIN);

    let widget = SettingsWidget::new(settings_with(450, NOW_PLAYING_THUMBNAIL_MAX + THUMBNAIL_STEP));
    assert_eq!(
        widget.clamped_now_playing_thumbnail_size(),
        NOW_PLAYING_THUMBNAIL_MAX
    );
}

#[test]
fn in_range_sizes_pass_through_unchanged() {
    let widget = SettingsWidget::new(settings_with(450, 1000));
    assert_eq!(widget.clamped_grid_thumbnail_size(), 450);
    assert_eq!(widget.clamped_now_playing_thumbnail_size(), 1000);
}

#[test]
fn clamped_values_follow_updated_settings() {
    let mut widget = SettingsWidget::new(settings_with(500, 500));
    assert_eq!(widget.clamped_grid_thumbnail_size(), 500);

    widget.set_settings(settings_with(50, 3000));
    assert_eq!(widget.clamped_grid_thumbnail_size(), GRID_THUMBNAIL_MIN);
    assert_eq!(
        widget.clamped_now_playing_thumbnail_size(),
        NOW_PLAYING_THUMBNAIL_MAX
    );
}

#[test]
fn view_builds_with_out_of_range_configured_sizes() {
    let widget = SettingsWidget::new(settings_with(0, u32::MAX));
    let _ = widget.view();

    let widget = SettingsWidget::new(settings_with(u32::MAX, 0));
    let _ = widget.view();
}

#[test]
fn slider_step_reaches_both_ends_of_each_range() {
    assert_eq!((GRID_THUMBNAIL_MAX - GRID_THUMBNAIL_MIN) % THUMBNAIL_STEP, 0);
    assert_eq!(
        (NOW_PLAYING_THUMBNAIL_MAX - NOW_PLAYING_THUMBNAIL_MIN) % THUMBNAIL_STEP,
        0
    );
}
