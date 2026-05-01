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

use plinth::config::{AppSettings, ColorScheme};
use iced::Theme;

#[test]
fn colorscheme_to_theme_and_back() {
    for scheme in ColorScheme::all_variants() {
        let theme: Theme = (*scheme).into();
        let back: ColorScheme = theme.into();
        assert_eq!(back, *scheme, "Roundtrip failed for {:?}", scheme);
    }
}

#[test]
fn colorscheme_all_variants_count() {
    assert_eq!(ColorScheme::all_variants().len(), 20);
}

#[test]
fn colorscheme_labels_unique() {
    let variants = ColorScheme::all_variants();
    let mut labels: Vec<_> = variants.iter().map(|s| s.label()).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), variants.len());
}

#[test]
fn appsettings_default_values() {
    let settings = AppSettings::default();
    assert_eq!(settings.grid_thumbnail_size, 450);
    assert_eq!(settings.now_playing_thumbnail_size, 1000);
    assert_eq!(settings.colorscheme, ColorScheme::CatppuccinMocha);
}

#[test]
fn appsettings_theme_delegation() {
    let settings = AppSettings::default();
    let theme: Theme = settings.colorscheme.into();
    assert_eq!(settings.theme(), theme);
}

#[test]
fn theme_unknown_variant_defaults_to_catppuccin_mocha() {
    let theme = Theme::Dark;
    let scheme: ColorScheme = theme.into();
    assert_eq!(scheme, ColorScheme::Dark);
}