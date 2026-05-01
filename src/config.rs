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

use std::fs;
use std::path::PathBuf;

use crate::mpd::types::AlbumSortConfig;
use iced::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
}

impl From<ColorScheme> for Theme {
    fn from(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Theme::Light,
            ColorScheme::Dark => Theme::Dark,
            ColorScheme::Dracula => Theme::Dracula,
            ColorScheme::Nord => Theme::Nord,
            ColorScheme::SolarizedLight => Theme::SolarizedLight,
            ColorScheme::SolarizedDark => Theme::SolarizedDark,
            ColorScheme::GruvboxLight => Theme::GruvboxLight,
            ColorScheme::GruvboxDark => Theme::GruvboxDark,
            ColorScheme::CatppuccinLatte => Theme::CatppuccinLatte,
            ColorScheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            ColorScheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            ColorScheme::CatppuccinMocha => Theme::CatppuccinMocha,
            ColorScheme::TokyoNight => Theme::TokyoNight,
            ColorScheme::TokyoNightStorm => Theme::TokyoNightStorm,
            ColorScheme::TokyoNightLight => Theme::TokyoNightLight,
            ColorScheme::KanagawaWave => Theme::KanagawaWave,
            ColorScheme::KanagawaDragon => Theme::KanagawaDragon,
            ColorScheme::KanagawaLotus => Theme::KanagawaLotus,
            ColorScheme::Moonfly => Theme::Moonfly,
            ColorScheme::Nightfly => Theme::Nightfly,
        }
    }
}

impl From<Theme> for ColorScheme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => ColorScheme::Light,
            Theme::Dark => ColorScheme::Dark,
            Theme::Dracula => ColorScheme::Dracula,
            Theme::Nord => ColorScheme::Nord,
            Theme::SolarizedLight => ColorScheme::SolarizedLight,
            Theme::SolarizedDark => ColorScheme::SolarizedDark,
            Theme::GruvboxLight => ColorScheme::GruvboxLight,
            Theme::GruvboxDark => ColorScheme::GruvboxDark,
            Theme::CatppuccinLatte => ColorScheme::CatppuccinLatte,
            Theme::CatppuccinFrappe => ColorScheme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato => ColorScheme::CatppuccinMacchiato,
            Theme::CatppuccinMocha => ColorScheme::CatppuccinMocha,
            Theme::TokyoNight => ColorScheme::TokyoNight,
            Theme::TokyoNightStorm => ColorScheme::TokyoNightStorm,
            Theme::TokyoNightLight => ColorScheme::TokyoNightLight,
            Theme::KanagawaWave => ColorScheme::KanagawaWave,
            Theme::KanagawaDragon => ColorScheme::KanagawaDragon,
            Theme::KanagawaLotus => ColorScheme::KanagawaLotus,
            Theme::Moonfly => ColorScheme::Moonfly,
            Theme::Nightfly => ColorScheme::Nightfly,
            _ => ColorScheme::CatppuccinMocha,
        }
    }
}

impl ColorScheme {
    pub fn all_variants() -> &'static [ColorScheme] {
        &[
            ColorScheme::CatppuccinMocha,
            ColorScheme::CatppuccinMacchiato,
            ColorScheme::CatppuccinFrappe,
            ColorScheme::CatppuccinLatte,
            ColorScheme::TokyoNight,
            ColorScheme::TokyoNightStorm,
            ColorScheme::TokyoNightLight,
            ColorScheme::KanagawaWave,
            ColorScheme::KanagawaDragon,
            ColorScheme::KanagawaLotus,
            ColorScheme::Moonfly,
            ColorScheme::Nightfly,
            ColorScheme::Dracula,
            ColorScheme::Nord,
            ColorScheme::GruvboxDark,
            ColorScheme::GruvboxLight,
            ColorScheme::SolarizedDark,
            ColorScheme::SolarizedLight,
            ColorScheme::Dark,
            ColorScheme::Light,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ColorScheme::Light => "Light",
            ColorScheme::Dark => "Dark",
            ColorScheme::Dracula => "Dracula",
            ColorScheme::Nord => "Nord",
            ColorScheme::SolarizedLight => "Solarized Light",
            ColorScheme::SolarizedDark => "Solarized Dark",
            ColorScheme::GruvboxLight => "Gruvbox Light",
            ColorScheme::GruvboxDark => "Gruvbox Dark",
            ColorScheme::CatppuccinLatte => "Catppuccin Latte",
            ColorScheme::CatppuccinFrappe => "Catppuccin Frappe",
            ColorScheme::CatppuccinMacchiato => "Catppuccin Macchiato",
            ColorScheme::CatppuccinMocha => "Catppuccin Mocha",
            ColorScheme::TokyoNight => "Tokyo Night",
            ColorScheme::TokyoNightStorm => "Tokyo Night Storm",
            ColorScheme::TokyoNightLight => "Tokyo Night Light",
            ColorScheme::KanagawaWave => "Kanagawa Wave",
            ColorScheme::KanagawaDragon => "Kanagawa Dragon",
            ColorScheme::KanagawaLotus => "Kanagawa Lotus",
            ColorScheme::Moonfly => "Moonfly",
            ColorScheme::Nightfly => "Nightfly",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub grid_thumbnail_size: u32,
    pub now_playing_thumbnail_size: u32,
    pub colorscheme: ColorScheme,
    pub sort_config: AlbumSortConfig,
}

impl AppSettings {
    pub fn theme(&self) -> Theme {
        self.colorscheme.into()
    }

    pub fn load() -> Self {
        let path = get_config_path();
        if let Ok(contents) = fs::read_to_string(&path)
            && let Ok(settings) = toml::from_str(&contents)
        {
            return settings;
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(serialized) = toml::to_string_pretty(self) {
            let _ = fs::write(&path, serialized);
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            grid_thumbnail_size: 450,
            now_playing_thumbnail_size: 1000,
            colorscheme: ColorScheme::CatppuccinMocha,
            sort_config: AlbumSortConfig::default(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("plinth")
        .join("settings.toml")
}