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

use crate::config::AppSettings;
use crate::ui::main_window::View;
use crate::ui::types::{Control, Message};
use iced::{
    Background, Color, Element, Length, Padding, Point, Rectangle, Size, Task,
    widget::{canvas, container, mouse_area, row, text},
};
use iced::widget::canvas::{Action, Event as CanvasEvent, Frame, Geometry, Path, Program};
use mpd::State;

struct ProgressBar {
    ratio: f32,
    accent: Color,
    track_color: Color,
}

impl Program<Message> for ProgressBar {
    type State = (f32, bool);

    fn update(
        &self,
        state: &mut Self::State,
        event: &CanvasEvent,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<Action<Message>> {
        let is_over = cursor.is_over(bounds);
        let (ratio, was_over) = state;

        match event {
            CanvasEvent::Mouse(mouse_event) => {
                use iced::mouse::Event as MouseEvent;
                match mouse_event {
                    MouseEvent::CursorMoved { position: _ } => {
                        if let iced::mouse::Cursor::Available(pos) = cursor {
                            let x = pos.x - bounds.x;
                            state.0 = (x / bounds.width).clamp(0.0, 1.0);
                        }
                        state.1 = is_over;
                        None
                    }
                    MouseEvent::ButtonPressed(iced::mouse::Button::Left) => {
                        if is_over && *was_over {
                            Some(Action::publish(Message::ProgressSeek(*ratio)).and_capture())
                        } else {
                            state.1 = is_over;
                            None
                        }
                    }
                    MouseEvent::ButtonReleased(iced::mouse::Button::Left) => {
                        state.1 = false;
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &(f32, bool),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry<iced::Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let track_height = 3.0;
        let y = (bounds.height - track_height) / 2.0;

        let track = Path::rectangle(
            Point::new(bounds.x, y),
            Size::new(bounds.width, track_height),
        );
        frame.fill(&track, self.track_color);

        let fill_width = (bounds.width * self.ratio.min(1.0).max(0.0)).round();
        if fill_width > 0.0 {
            let fill_path = Path::rectangle(
                Point::new(bounds.x, y),
                Size::new(fill_width, track_height),
            );
            frame.fill(&fill_path, self.accent);
        }

        let radius = 5.0;
        let cap_y = y + track_height / 2.0;
        let cap_x = bounds.x + bounds.width * self.ratio.min(1.0).max(0.0);
        if self.ratio > 0.0 && self.ratio < 1.0 {
            let circle = Path::circle(Point::new(cap_x, cap_y), radius);
            frame.fill(&circle, self.accent);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &(f32, bool),
        _bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if matches!(cursor, iced::mouse::Cursor::Available(_)) {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

pub struct PlayerControl {
    hovered: Option<Control>,
}

impl PlayerControl {
    pub fn new() -> Self {
        Self {
            hovered: None,
        }
    }

    pub fn set_hovered(&mut self, hovered: Option<Control>) {
        self.hovered = hovered;
    }

    fn control_button(
        &self,
        icon: &'static str,
        control: Control,
        message: Message,
        hover_bg: Color,
        normal_text: Color,
        hover_text: Color,
    ) -> Element<'static, Message> {
        let hovered = self.hovered == Some(control);
        let background = if hovered {
            hover_bg
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.0)
        };
        let text_color = if hovered { hover_text } else { normal_text };

        mouse_area(container(text(icon).size(22)).padding(8).style(move |_| {
            container::Style::default()
                .background(Background::Color(background))
                .color(text_color)
        }))
        .on_enter(Message::ControlHovered(control))
        .on_exit(Message::ControlUnhovered(control))
        .on_press(message)
        .into()
    }

    fn control_button_disabled(
        &self,
        icon: &'static str,
        control: Control,
        message: Message,
        hover_bg: Color,
        normal_text: Color,
        hover_text: Color,
        disabled: bool,
        disabled_text: Color,
    ) -> Element<'static, Message> {
        let hovered = self.hovered == Some(control) && !disabled;
        let background = if hovered {
            hover_bg
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.0)
        };
        let text_color = if disabled {
            disabled_text
        } else if hovered {
            hover_text
        } else {
            normal_text
        };

        mouse_area(container(text(icon).size(22)).padding(8).style(move |_| {
            container::Style::default()
                .background(Background::Color(background))
                .color(text_color)
        }))
        .on_enter(Message::ControlHovered(control))
        .on_exit(Message::ControlUnhovered(control))
        .on_press(message)
        .into()
    }

    pub fn view(&self, playback_state: State, elapsed_secs: Option<f64>, duration_secs: Option<f64>, settings: &AppSettings, current_view: View, is_reload_pending: bool) -> Element<'_, Message> {
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let hover_bg = extended.background.weak.color;
        let normal_text = extended.secondary.weak.color;
        let hover_text = extended.background.base.text;
        let accent = extended.primary.strong.color;
        let disabled_text = Color::from_rgba(
            extended.secondary.weak.color.r,
            extended.secondary.weak.color.g,
            extended.secondary.weak.color.b,
            0.4,
        );

        let ratio = match (elapsed_secs, duration_secs) {
            (Some(elapsed), Some(duration)) if duration > 0.0 => (elapsed / duration) as f32,
            _ => 0.0,
        };

        let track_color = Color::from_rgba(0.75, 0.75, 0.75, 0.15);

        let progress_bar = ProgressBar {
            ratio,
            accent,
            track_color,
        };

        let progress = canvas(progress_bar)
            .width(Length::Fill)
            .height(Length::Fixed(12.0));

        let left_controls = row![
            self.control_button(
                "▦",
                Control::AlbumGrid,
                Message::AlbumGridButtonClicked,
                hover_bg,
                normal_text,
                hover_text,
            ),
            self.control_button(
                "♬",
                Control::NowPlaying,
                Message::NowPlayingButtonClicked,
                hover_bg,
                normal_text,
                hover_text,
            ),
            self.control_button(
                "⚙",
                Control::Settings,
                Message::SettingsButtonClicked,
                hover_bg,
                normal_text,
                hover_text,
            ),
        ]
        .spacing(6)
        .width(Length::FillPortion(1));

        let play_icon = if playback_state == State::Play {
            "❚❚"
        } else {
            "▶"
        };

        let transport_controls = row![
            self.control_button(
                "|◀",
                Control::PreviousSong,
                Message::PreviousSong,
                hover_bg,
                normal_text,
                hover_text,
            ),
            self.control_button(
                play_icon,
                Control::PlayPause,
                Message::PlayPause,
                hover_bg,
                normal_text,
                hover_text,
            ),
            self.control_button(
                "▶|",
                Control::NextSong,
                Message::NextSong,
                hover_bg,
                normal_text,
                hover_text,
            ),
        ]
        .spacing(6);

        let right_controls = if current_view == View::NowPlaying {
            row![
                container(text("")).width(Length::Fill),
                self.control_button(
                    "🗑",
                    Control::ClearQueue,
                    Message::ClearQueue,
                    hover_bg,
                    normal_text,
                    hover_text,
                ),
            ]
            .spacing(0)
            .width(Length::FillPortion(1))
            .align_y(iced::Alignment::Center)
        } else if current_view == View::AlbumGrid {
             let (reload_icon, disabled) = if is_reload_pending { ("◷", true) } else { ("↻", false) };
            row![
                container(text("")).width(Length::Fill),
                self.control_button_disabled(
                    reload_icon,
                    Control::ReloadDatabase,
                    Message::ReloadDatabase,
                    hover_bg,
                    normal_text,
                    hover_text,
                    disabled,
                    disabled_text,
                ),
            ]
            .spacing(0)
            .width(Length::FillPortion(1))
            .align_y(iced::Alignment::Center)
        } else {
            row![container(text("")).width(Length::Fill)].spacing(0).width(Length::FillPortion(1)).into()
        };

        iced::widget::column![
            container(progress)
                .width(Length::Fill)
                .padding(Padding { left: 0.0, right: 0.0, top: 0.0, bottom: 2.0 }),
            row![
                left_controls,
                container(transport_controls)
                    .width(Length::FillPortion(1))
                    .center_x(Length::Fill),
                right_controls,
            ]
            .spacing(10)
            .padding(Padding { left: 6.0, right: 6.0, top: 2.0, bottom: 6.0 })
            .width(Length::Fill),
        ]
        .spacing(0)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ControlHovered(control) => {
                self.set_hovered(Some(control));
                Task::none()
            }
            Message::ControlUnhovered(_control) => {
                self.set_hovered(None);
                Task::none()
            }
            _ => Task::none()
        }
    }
}