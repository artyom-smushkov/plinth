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
    border::radius,
    widget::{canvas, container, mouse_area, row, text},
};
use iced::widget::canvas::{
    Action, Event as CanvasEvent, Frame, Geometry, LineCap, LineJoin, Path, Program, Stroke,
    path::Builder,
};
use lyon_path::math::Transform;
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
            Point::new(0.0, y),
            Size::new(bounds.width, track_height),
        );
        frame.fill(&track, self.track_color);

        let fill_width = (bounds.width * self.ratio.clamp(0.0, 1.0)).round();
        if fill_width > 0.0 {
            let fill_path = Path::rectangle(
                Point::new(0.0, y),
                Size::new(fill_width, track_height),
            );
            frame.fill(&fill_path, self.accent);
        }

        let radius = 5.0;
        let cap_y = y + track_height / 2.0;
        let cap_x = bounds.width * self.ratio.clamp(0.0, 1.0);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    AlbumGrid,
    NowPlaying,
    Settings,
    Previous,
    Play,
    Pause,
    Next,
    Close,
    Reload,
}

const ICON_DESIGN_SIZE: f32 = 24.0;
const ICON_STROKE_WIDTH: f32 = 1.5;

impl Icon {
    pub fn all() -> &'static [Icon] {
        &[
            Icon::AlbumGrid,
            Icon::NowPlaying,
            Icon::Settings,
            Icon::Previous,
            Icon::Play,
            Icon::Pause,
            Icon::Next,
            Icon::Close,
            Icon::Reload,
        ]
    }

    pub fn paths(&self) -> Vec<Path> {
        match self {
            Icon::AlbumGrid => vec![
                Path::rounded_rectangle(Point::new(3.0, 3.0), Size::new(7.0, 7.0), radius(1.5)),
                Path::rounded_rectangle(Point::new(14.0, 3.0), Size::new(7.0, 7.0), radius(1.5)),
                Path::rounded_rectangle(Point::new(3.0, 14.0), Size::new(7.0, 7.0), radius(1.5)),
                Path::rounded_rectangle(Point::new(14.0, 14.0), Size::new(7.0, 7.0), radius(1.5)),
            ],
            Icon::NowPlaying => vec![
                Path::new(|p| {
                    p.move_to(Point::new(9.0, 18.0));
                    p.line_to(Point::new(9.0, 5.0));
                    p.line_to(Point::new(21.0, 3.0));
                    p.line_to(Point::new(21.0, 16.0));
                }),
                Path::circle(Point::new(6.0, 18.0), 3.0),
                Path::circle(Point::new(18.0, 16.0), 3.0),
            ],
            Icon::Settings => {
                let center = Point::new(12.0, 12.0);
                let outline = Path::new(|p| {
                    p.move_to(polar(center, 9.8, 0.0));
                    for i in 0..8 {
                        let tooth_start = (i as f32) * 45.0f32.to_radians();
                        arc_bezier(p, center, 9.8, tooth_start, tooth_start + 16.0f32.to_radians());
                        p.line_to(polar(center, 7.0, tooth_start + 22.5f32.to_radians()));
                        arc_bezier(
                            p,
                            center,
                            7.0,
                            tooth_start + 22.5f32.to_radians(),
                            tooth_start + 38.5f32.to_radians(),
                        );
                        p.line_to(polar(center, 9.8, tooth_start + 45.0f32.to_radians()));
                    }
                    p.close();
                });
                vec![outline, Path::circle(center, 3.2)]
            }
            Icon::Previous => vec![
                Path::new(|p| {
                    p.move_to(Point::new(19.0, 4.0));
                    p.line_to(Point::new(19.0, 20.0));
                    p.line_to(Point::new(9.0, 12.0));
                    p.close();
                }),
                Path::line(Point::new(5.0, 5.0), Point::new(5.0, 19.0)),
            ],
            Icon::Play => vec![Path::new(|p| {
                p.move_to(Point::new(6.0, 3.0));
                p.line_to(Point::new(20.0, 12.0));
                p.line_to(Point::new(6.0, 21.0));
                p.close();
            })],
            Icon::Pause => vec![
                Path::rounded_rectangle(Point::new(6.0, 4.0), Size::new(4.0, 16.0), radius(1.0)),
                Path::rounded_rectangle(Point::new(14.0, 4.0), Size::new(4.0, 16.0), radius(1.0)),
            ],
            Icon::Next => vec![
                Path::new(|p| {
                    p.move_to(Point::new(5.0, 4.0));
                    p.line_to(Point::new(15.0, 12.0));
                    p.line_to(Point::new(5.0, 20.0));
                    p.close();
                }),
                Path::line(Point::new(19.0, 5.0), Point::new(19.0, 19.0)),
            ],
            Icon::Close => vec![
                Path::line(Point::new(6.0, 6.0), Point::new(18.0, 18.0)),
                Path::line(Point::new(18.0, 6.0), Point::new(6.0, 18.0)),
            ],
            Icon::Reload => vec![
                Path::new(|p| {
                    let center = Point::new(12.0, 12.0);
                    p.move_to(Point::new(21.0, 12.0));
                    arc_bezier(p, center, 9.0, 0.0, 1.5 * std::f32::consts::PI);
                    p.bezier_curve_to(
                        Point::new(14.52, 3.0),
                        Point::new(16.93, 4.0),
                        Point::new(18.74, 5.74),
                    );
                    p.line_to(Point::new(21.0, 8.0));
                }),
                Path::new(|p| {
                    p.move_to(Point::new(21.0, 3.0));
                    p.line_to(Point::new(21.0, 8.0));
                    p.line_to(Point::new(16.0, 8.0));
                }),
            ],
        }
    }
}

fn polar(center: Point, radius: f32, angle: f32) -> Point {
    Point::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
}

fn arc_bezier(p: &mut Builder, center: Point, radius: f32, start: f32, end: f32) {
    let total = end - start;
    let segments = (total / (std::f32::consts::PI / 2.0)).ceil().max(1.0) as usize;
    let step = total / segments as f32;
    let mut angle = start;
    for _ in 0..segments {
        let next = angle + step;
        let k = (4.0 / 3.0) * (step / 4.0).tan() * radius;
        let control_a = Point::new(
            center.x + radius * angle.cos() - k * angle.sin(),
            center.y + radius * angle.sin() + k * angle.cos(),
        );
        let control_b = Point::new(
            center.x + radius * next.cos() + k * next.sin(),
            center.y + radius * next.sin() - k * next.cos(),
        );
        p.bezier_curve_to(control_a, control_b, polar(center, radius, next));
        angle = next;
    }
}

pub(crate) struct ControlIcon {
    pub(crate) icon: Icon,
    pub(crate) color: Color,
}

impl Program<Message> for ControlIcon {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &CanvasEvent,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Option<Action<Message>> {
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry<iced::Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let scale = (bounds.width.min(bounds.height) / ICON_DESIGN_SIZE).max(0.0);
        let origin = Point::new(
            (bounds.width - ICON_DESIGN_SIZE * scale) / 2.0,
            (bounds.height - ICON_DESIGN_SIZE * scale) / 2.0,
        );
        let transform = Transform::translation(origin.x, origin.y).pre_scale(scale, scale);
        let stroke = Stroke::default()
            .with_width(ICON_STROKE_WIDTH * scale)
            .with_color(self.color)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round);

        for path in self.icon.paths() {
            frame.stroke(&path.transform(&transform), stroke);
        }

        vec![frame.into_geometry()]
    }
}

struct ButtonStyle {
    hover_bg: Color,
    normal_text: Color,
    hover_text: Color,
    disabled_text: Color,
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

    pub fn hovered_control(&self) -> Option<Control> {
        self.hovered
    }

    pub fn icon_for(&self, control: Control, playback_state: State) -> Icon {
        match control {
            Control::AlbumGrid => Icon::AlbumGrid,
            Control::NowPlaying => Icon::NowPlaying,
            Control::Settings => Icon::Settings,
            Control::PreviousSong => Icon::Previous,
            Control::PlayPause => {
                if playback_state == State::Play {
                    Icon::Pause
                } else {
                    Icon::Play
                }
            }
            Control::NextSong => Icon::Next,
            Control::ClearQueue => Icon::Close,
            Control::ReloadDatabase => Icon::Reload,
        }
    }

    fn control_button(
        &self,
        icon: Icon,
        control: Control,
        message: Message,
        style: &ButtonStyle,
        disabled: bool,
    ) -> Element<'static, Message> {
        let is_hovered = self.hovered == Some(control) && !disabled;
        let background = if is_hovered {
            style.hover_bg
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.0)
        };
        let icon_color = if disabled {
            style.disabled_text
        } else if is_hovered {
            style.hover_text
        } else {
            style.normal_text
        };

        let icon_widget = canvas(ControlIcon {
            icon,
            color: icon_color,
        })
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(22.0));

        mouse_area(container(icon_widget).padding(8).style(move |_| {
            container::Style::default().background(Background::Color(background))
        }))
        .on_enter(Message::ControlHovered(control))
        .on_exit(Message::ControlUnhovered(control))
        .on_press(message)
        .into()
    }

    pub fn view(&self, playback_state: State, elapsed_secs: Option<f64>, duration_secs: Option<f64>, settings: &AppSettings, current_view: View, is_reload_pending: bool) -> Element<'_, Message> {
        let theme = settings.theme();
        let extended = theme.extended_palette();
        let accent = extended.primary.strong.color;
        let style = ButtonStyle {
            hover_bg: extended.background.weak.color,
            normal_text: extended.secondary.weak.color,
            hover_text: extended.background.base.text,
            disabled_text: Color::from_rgba(
                extended.secondary.weak.color.r,
                extended.secondary.weak.color.g,
                extended.secondary.weak.color.b,
                0.4,
            ),
        };

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
            self.control_button(Icon::AlbumGrid, Control::AlbumGrid, Message::AlbumGridButtonClicked, &style, false),
            self.control_button(Icon::NowPlaying, Control::NowPlaying, Message::NowPlayingButtonClicked, &style, false),
            self.control_button(Icon::Settings, Control::Settings, Message::SettingsButtonClicked, &style, false),
        ]
        .spacing(6)
        .width(Length::FillPortion(1));

        let transport_controls = row![
            self.control_button(Icon::Previous, Control::PreviousSong, Message::PreviousSong, &style, false),
            self.control_button(self.icon_for(Control::PlayPause, playback_state), Control::PlayPause, Message::PlayPause, &style, false),
            self.control_button(Icon::Next, Control::NextSong, Message::NextSong, &style, false),
        ]
        .spacing(6);

        let right_controls = if current_view == View::NowPlaying {
            row![
                container(text("")).width(Length::Fill),
                self.control_button(Icon::Close, Control::ClearQueue, Message::ClearQueue, &style, false),
            ]
            .spacing(0)
            .width(Length::FillPortion(1))
            .align_y(iced::Alignment::Center)
        } else if current_view == View::AlbumGrid {
            row![
                container(text("")).width(Length::Fill),
                self.control_button(Icon::Reload, Control::ReloadDatabase, Message::ReloadDatabase, &style, is_reload_pending),
            ]
            .spacing(0)
            .width(Length::FillPortion(1))
            .align_y(iced::Alignment::Center)
        } else {
            row![container(text("")).width(Length::Fill)].spacing(0).width(Length::FillPortion(1))
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
