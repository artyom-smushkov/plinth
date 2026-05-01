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

use std::rc::Rc;

use super::album::{AlbumDisplayOption, AlbumWidget};
use crate::mpd::types::{Album, AlbumSortConfig};
use crate::ui::types::Message;
use iced::{
    widget::{column, container, grid, rule::horizontal, scrollable, text},
    Alignment, Element, Length, Padding, Task,
};

struct Group {
    label: String,
    widgets: Vec<AlbumWidget>,
}

pub struct AlbumGrid {
    groups: Vec<Group>,
    albums: Rc<Vec<Rc<Album>>>,
    albums_order: Rc<Vec<usize>>,
    sort_config: AlbumSortConfig,
    grid_thumbnail_size: u32,
}

impl AlbumGrid {
    pub fn new(
        albums: Rc<Vec<Rc<Album>>>,
        albums_order: Rc<Vec<usize>>,
        sort_config: &AlbumSortConfig,
        grid_thumbnail_size: u32,
    ) -> Self {
        let groups = Self::build_groups(&albums, &albums_order, sort_config, grid_thumbnail_size);

        Self {
            groups,
            albums,
            albums_order,
            sort_config: sort_config.clone(),
            grid_thumbnail_size,
        }
    }

    fn build_groups(
        albums: &Rc<Vec<Rc<Album>>>,
        albums_order: &Rc<Vec<usize>>,
        sort_config: &AlbumSortConfig,
        grid_thumbnail_size: u32,
    ) -> Vec<Group> {
        let mut groups: Vec<Group> = Vec::new();

        for &i in albums_order.iter() {
            let album = &albums[i];

            if let Some(field) = sort_config.highest {
                let label = album.group_label(field).to_string();
                let mut found_idx = None;
                for (idx, group) in groups.iter_mut().enumerate() {
                    if group.label == label {
                        found_idx = Some(idx);
                        break;
                    }
                }
                match found_idx {
                    Some(idx) => {
                        groups[idx].widgets
                            .push(AlbumWidget::new(album.clone(), Some(i), AlbumDisplayOption::Grid, grid_thumbnail_size));
                    }
                    None => {
                        groups.push(Group {
                            label,
                            widgets: vec![AlbumWidget::new(album.clone(), Some(i), AlbumDisplayOption::Grid, grid_thumbnail_size)],
                        });
                    }
                }
            } else {
                if groups.is_empty() {
                    groups.push(Group {
                        label: String::new(),
                        widgets: vec![AlbumWidget::new(album.clone(), Some(i), AlbumDisplayOption::Grid, grid_thumbnail_size)],
                    });
                } else {
                    groups[0].widgets
                        .push(AlbumWidget::new(album.clone(), Some(i), AlbumDisplayOption::Grid, grid_thumbnail_size));
                }
            }
        }

        groups
    }

    pub fn rebuild(&mut self, sort_config: &AlbumSortConfig, grid_thumbnail_size: u32) -> Task<Message> {
        if sort_config == &self.sort_config && grid_thumbnail_size == self.grid_thumbnail_size {
            return Task::none();
        }
        self.sort_config = sort_config.clone();
        self.grid_thumbnail_size = grid_thumbnail_size;
        self.groups = Self::build_groups(&self.albums, &self.albums_order, sort_config, grid_thumbnail_size);
        self.init()
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::batch(
            self.groups
                .iter_mut()
                .flat_map(|group| group.widgets.iter_mut())
                .map(|widget| widget.init())
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let settings = crate::config::AppSettings::default();
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let faded_color = extended.secondary.weak.color;

        let mut content = column![];

        for group in &self.groups {
            let mut group_content = column![];

            if !group.label.is_empty() {
                let header = iced::widget::row![
                    text(&group.label).size(18).color(faded_color),
                    container(horizontal(1)).width(Length::Fill),
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                group_content = group_content.push(container(header).width(Length::Fill).padding(Padding {
                    bottom: 10.0,
                    ..Padding::ZERO
                }));
            }

            let grid = group
                .widgets
                .iter()
                .fold(grid![], |col, widget| col.push(widget.view()))
                .fluid(self.grid_thumbnail_size)
                .spacing(10);

            group_content = group_content.push(grid);

            content = content.push(container(group_content).padding(20.0));
        }

        scrollable(content).into()
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn total_widget_count(&self) -> usize {
        self.groups.iter().map(|g| g.widgets.len()).sum()
    }

    pub fn get_widget_thumbnail_loaded(&self, group_idx: usize, widget_idx: usize) -> Option<bool> {
        self.groups.get(group_idx).and_then(|g| {
            g.widgets.get(widget_idx).map(|w| w.is_thumbnail_loaded())
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GridAlbumClicked(index) => {
                Task::done(Message::GridAlbumClicked(index))
            }
            Message::GridThumbnailReady(widget_id, result) => {
                for group in self.groups.iter_mut() {
                    for widget in group.widgets.iter_mut() {
                        if widget.widget_id == widget_id {
                            let _ = widget.update(Message::GridThumbnailReady(widget_id, result));
                            return Task::none();
                        }
                    }
                }
                Task::none()
            }
            _ => Task::none()
        }
    }
}