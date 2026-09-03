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

pub mod album;
pub mod album_grid;
mod album_info;
pub mod main_window;
pub mod now_playing;
pub mod player_control;
pub mod playing_album;
pub mod settings;
pub mod track;
pub mod types;

use crate::config::AppSettings;
use crate::mpd::{PlaybackClient, ReloadFlag, playback::PlaybackClientError};
use crate::ui::types::Message;
use crate::ui::main_window::MainWindow;
use std::rc::Rc;
use iced::{
    event,
    keyboard::{self, key::Named, Key},
    widget::{button, Column, Container, Text},
    window, Alignment, Element, Event, Font, Length, Subscription, Task, Theme,
};
use mpd::State;

pub(crate) const UI_FONT: Font = Font::with_name("Adwaita Sans");

pub struct App {
    pub state: AppState,
    pub settings: AppSettings,
    pub reload_flag: ReloadFlag,
}

pub enum AppState {
    Error(PlaybackClientError),
    Running {
        main_window: Box<MainWindow>,
        playback_client: PlaybackClient,
        error: Option<PlaybackClientError>,
        last_elapsed: Option<f64>,
        last_duration: Option<f64>,
        last_state: State,
        last_song_position: Option<usize>,
        updating_db: Option<u32>,
        pending_reload: bool,
    },
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load();
        let reload_flag = ReloadFlag::new();
        match PlaybackClient::new() {
            Ok(mut playback_client) => {
                let _ = playback_client.init_queue_from_mpd();
                let albums_order = playback_client.albums_order.clone();
                let sort_config = playback_client.sort_config.clone();
                let main_window = MainWindow::new(
                    playback_client.albums.clone(),
                    albums_order,
                    sort_config,
                    settings.clone(),
                );
                let init_playback_state = playback_client
                    .playback_state()
                    .unwrap_or(::mpd::State::Stop);

                let mut main_window = main_window;
                let window_init_task = main_window.init();

                let queue_sync_task = if !playback_client.queue.borrow().is_empty() {
                    Task::done(Message::SyncQueue(playback_client.queue.clone()))
                } else {
                    Task::none()
                };

               (
                    Self {
                        state: AppState::Running {
                            main_window: Box::new(main_window),
                            playback_client,
                            error: None,
                            last_elapsed: None,
                            last_duration: None,
                            last_state: init_playback_state,
                            last_song_position: None,
                            updating_db: None,
                            pending_reload: false,
                        },
                        settings,
                        reload_flag,
                    },
                    Task::batch(vec![window_init_task, queue_sync_task]),
                )
            }
            Err(e) => (
                Self {
                    state: AppState::Error(e),
                    settings,
                    reload_flag: ReloadFlag::new(),
                },
                Task::none(),
            ),
        }
    }

    pub fn with_test_client(playback_client: PlaybackClient) -> (Self, Task<Message>) {
        let settings = AppSettings::load();
        let albums_order = playback_client.albums_order.clone();
        let sort_config = playback_client.sort_config.clone();
        let mut main_window = MainWindow::new(
            playback_client.albums.clone(),
            albums_order,
            sort_config,
            settings.clone(),
        );
        let window_init_task = main_window.init();

        let queue_sync_task = if !playback_client.queue.borrow().is_empty() {
            Task::done(Message::SyncQueue(playback_client.queue.clone()))
        } else {
            Task::none()
        };

       (
            Self {
                state: AppState::Running {
                    main_window: Box::new(main_window),
                    playback_client,
                    error: None,
                    last_elapsed: None,
                    last_duration: None,
                    last_state: State::Stop,
                    last_song_position: None,
                    updating_db: None,
                    pending_reload: false,
                },
                settings,
                reload_flag: ReloadFlag::new(),
            },
            Task::batch(vec![window_init_task, queue_sync_task]),
        )
    }

    pub fn main_window(&self) -> Option<&MainWindow> {
        match &self.state {
            AppState::Running { main_window, .. } => Some(&**main_window),
            _ => None,
        }
    }

    pub fn main_window_mut(&mut self) -> Option<&mut MainWindow> {
        match &mut self.state {
            AppState::Running { main_window, .. } => Some(&mut **main_window),
            _ => None,
        }
    }

    pub fn playback_client_mut(&mut self) -> Option<&mut PlaybackClient> {
        match &mut self.state {
            AppState::Running { playback_client, .. } => Some(playback_client),
            _ => None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &mut self.state {
            AppState::Error(_) => match message {
                Message::DismissError => Task::none(),
                Message::Exit => {
                    std::process::exit(1);
                }
                _ => Task::none(),
            },
            AppState::Running {
                main_window,
                playback_client,
                error,
                last_elapsed,
                last_duration,
                last_state,
                last_song_position,
                updating_db,
                pending_reload,
            } => match message {
                Message::DismissError => {
                    *error = None;
                    Task::none()
                }
                Message::Exit => {
                    std::process::exit(1);
                }
                Message::PlaybackStateUpdate { .. } => {
                    let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() else {
                        return Task::none();
                    };

                    let state_changed = state != *last_state || elapsed != *last_elapsed || duration != *last_duration;
                    let position_changed = song_pos != *last_song_position;

                    if !state_changed && !position_changed {
                        return Task::none();
                    }

                    *last_state = state;
                    *last_elapsed = elapsed;
                    *last_duration = duration;
                    *last_song_position = song_pos;

                    let set_state_task = Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos));

                    if position_changed
                        && let Some(pos) = song_pos
                        && let Ok(queue) = playback_client.queue.try_borrow()
                    {
                        let mut cumulative_pos = 0;
                        for (album_idx, &album_in_queue_idx) in queue.iter().enumerate() {
                            let album = &playback_client.albums[album_in_queue_idx];
                            let track_count = album.tracks.len();

                            if pos >= cumulative_pos && pos < cumulative_pos + track_count {
                                let track_index_in_album = pos - cumulative_pos;
                                return Task::batch(vec![
                                    set_state_task,
                                    main_window.update(Message::SetCurrentTrack(
                                        album_idx,
                                        track_index_in_album,
                                    )),
                                ]);
                            }
                            cumulative_pos += track_count;
                        }
                    }

                    let needs_rebuild = playback_client.queue.try_borrow().is_ok_and(|queue| {
                        if queue.is_empty() {
                            return false;
                        }
                        let pos_matches = if let Some(p) = song_pos {
                            Self::position_matches_queue(&playback_client.albums, &queue, p)
                        } else {
                            true
                        };
                        (song_pos.is_some() || state != ::mpd::State::Stop)
                            && (song_pos.is_none() || !pos_matches)
                    });

                    if needs_rebuild {
                        let _ = playback_client.init_queue_from_mpd();
                        return Task::batch(vec![
                            set_state_task,
                            Task::done(Message::SyncQueue(playback_client.queue.clone())),
                        ]);
                    }

                    set_state_task
                }
                Message::PlayPause => {
                    let _ = playback_client.toggle_play_pause();
                    if let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() {
                        return Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos));
                    }
                    Task::none()
                }
                Message::PreviousSong => {
                    let _ = playback_client.previous_song();
                    Task::none()
                }
                Message::SeekTo(seconds) => {
                    let _ = playback_client.seek_to(seconds);
                    Task::none()
                }
                Message::ProgressSeek(ratio) => {
                    if let Ok((Some(elapsed), Some(duration))) = playback_client.get_elapsed_duration()
                        && elapsed > 0.0 && duration > 0.0
                    {
                        let _ = playback_client.seek_to((ratio as f64) * duration);
                    }
                    Task::none()
                }
                Message::NextSong => {
                    let _ = playback_client.next_song();
                    Task::none()
                }
                Message::GridAlbumClicked(index) => {
                    let _ = playback_client.add_album_to_queue(index);
                    let playback_task =
                        if let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() {
                            Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos))
                        } else {
                            Task::none()
                        };
                    Task::batch(vec![
                        playback_task,
                        Task::done(Message::SyncQueue(playback_client.queue.clone())),
                    ])
                }
                Message::TrackClicked(queue_index, track_index) => {
                    handle_play_track(playback_client, queue_index, track_index)
                }
                Message::RemoveAlbumFromQueue(queue_index) => {
                    let _ = playback_client.remove_album_from_queue(queue_index);
                    let playback_task =
                        if let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() {
                            Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos))
                        } else {
                            Task::none()
                        };
                    Task::batch(vec![
                        playback_task,
                        Task::done(Message::SyncQueue(playback_client.queue.clone())),
                    ])
                }
                Message::ClearQueue => {
                    let _ = playback_client.clear_queue();
                    let playback_task =
                        if let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() {
                            Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos))
                        } else {
                            Task::none()
                        };
                    Task::batch(vec![
                        playback_task,
                        Task::done(Message::SyncQueue(playback_client.queue.clone())),
                    ])
                }
                Message::ReloadDatabase => {
                    if *pending_reload {
                        Task::none()
                    } else if let Err(e) = playback_client.reload_database() {
                        *error = Some(e);
                        Task::none()
                    } else {
                        self.reload_flag.set(true);
                        *pending_reload = true;
                        Task::none()
                    }
                }
                Message::UpdatingDatabase(new_state) => {
                    *updating_db = new_state;
                    if new_state.is_some() && !*pending_reload {
                        *pending_reload = true;
                    }
                    if new_state.is_none() {
                        *pending_reload = false;
                    }
                    Task::none()
                }
                Message::DatabaseUpdated => {
                    *pending_reload = false;
                    match playback_client.reload_albums() {
                        Ok(()) => {
                            let albums = playback_client.albums.clone();
                            let albums_order = playback_client.albums_order.clone();
                            let grid_task = main_window.update_albums(albums, albums_order);
                            let playback_task =
                                if let Ok((state, elapsed, duration, song_pos)) = playback_client.get_full_status() {
                                    Task::done(Message::SetPlaybackState(state, elapsed, duration, song_pos))
                                } else {
                                    Task::none()
                                };
                            Task::batch(vec![
                                grid_task,
                                playback_task,
                                Task::done(Message::SyncQueue(playback_client.queue.clone())),
                            ])
                        }
                        Err(e) => {
                            *error = Some(e);
                            Task::none()
                        }
                    }
                }
                Message::GridThumbnailReady(_, _)
                | Message::NowPlayingThumbnailReady(_, _)
                | Message::TrackHovered(_, _)
                | Message::TrackUnhovered(_, _)
                | Message::TrackStartedPlaying(_, _)
                | Message::TrackEndedPlaying(_, _)
                | Message::SetCurrentTrack(_, _)
                | Message::SyncQueue(_)
                | Message::AlbumHovered(_)
                | Message::AlbumUnhovered(_)
                | Message::ViewScrolled(_, _) => main_window.update(message),
                Message::SetPlaybackState(state, elapsed, duration, song_pos) => {
                    let mut tasks = vec![main_window.update(Message::SetPlaybackState(state, elapsed, duration, song_pos))];

                    if let Some(pos) = song_pos
                        && let Ok(queue) = playback_client.queue.try_borrow()
                    {
                        let mut cumulative_pos = 0;
                        for (album_idx, &album_in_queue_idx) in queue.iter().enumerate() {
                            let album = &playback_client.albums[album_in_queue_idx];
                            let track_count = album.tracks.len();

                            if pos >= cumulative_pos && pos < cumulative_pos + track_count {
                                let track_index_in_album = pos - cumulative_pos;
                                tasks.push(main_window.update(Message::SetCurrentTrack(
                                    album_idx,
                                    track_index_in_album,
                                )));
                                break;
                            }
                            cumulative_pos += track_count;
                        }
                    }

                    Task::batch(tasks)
                }
                Message::AlbumGridButtonClicked
                | Message::NowPlayingButtonClicked
                | Message::SettingsButtonClicked
                | Message::ControlHovered(_)
                | Message::ControlUnhovered(_) => main_window.update(message),
                Message::SettingGridThumbnailSizeChanged(size) => {
                    self.settings.grid_thumbnail_size = size;
                    self.settings.save();
                    main_window.update(message)
                }
                Message::SettingNowPlayingThumbnailSizeChanged(size) => {
                    self.settings.now_playing_thumbnail_size = size;
                    self.settings.save();
                    main_window.update(message)
                }
                Message::SettingColorschemeChanged(scheme) => {
                    self.settings.colorscheme = scheme;
                    self.settings.save();
                    main_window.apply_colorscheme(scheme);
                    Task::none()
                }
                Message::SettingSortHighestChanged(field) => {
                    self.settings.sort_config.highest = field;
                    self.settings.save();
                    main_window.update(message)
                }
                Message::SettingSortMiddleChanged(field) => {
                    self.settings.sort_config.middle = field;
                    self.settings.save();
                    main_window.update(message)
                }
                Message::SettingSortLowestChanged(field) => {
                    self.settings.sort_config.lowest = field;
                    self.settings.save();
                    main_window.update(message)
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Error(err) => view_error(err),
            AppState::Running {
                main_window, error, pending_reload, ..
            } => match error {
                Some(err) => view_error(err),
                None => main_window.view(*pending_reload),
            },
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            event::listen_with(Self::handle_event),
            crate::mpd::poller::polling_subscription(),
        ])
    }

    fn handle_event(event: Event, _status: event::Status, _id: window::Id) -> Option<Message> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                Key::Named(Named::Space) => Some(Message::PlayPause),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn theme(&self) -> Theme {
        self.settings.theme()
    }

    fn position_matches_queue(
        albums: &Rc<Vec<Rc<crate::mpd::types::Album>>>,
        queue: &[usize],
        pos: usize,
    ) -> bool {
        let mut cumulative = 0;
        for &album_idx in queue {
            let track_count = albums[album_idx].tracks.len();
            if pos >= cumulative && pos < cumulative + track_count {
                return true;
            }
            cumulative += track_count;
        }
        false
    }
}

fn handle_play_track(
    playback_client: &mut PlaybackClient,
    queue_index: usize,
    track_index: usize,
) -> Task<Message> {
    if playback_client.switch_to_track(queue_index, track_index).is_ok() {
        Task::done(Message::SetCurrentTrack(queue_index, track_index))
    } else {
        Task::none()
    }
}

fn view_error(err: &PlaybackClientError) -> Element<'_, Message> {
    let (error_text, dismiss_or_exit_button) = match err {
        PlaybackClientError::MPDConnectionError(msg) => (
            format!("Could not connect to mpd: {}", msg),
            button(Text::new("exit")).on_press(Message::Exit),
        ),
        PlaybackClientError::MPDRetrieveError(msg) => (
            format!("Could not retrieve data from mpd: {}", msg),
            button(Text::new("exit")).on_press(Message::Exit),
        ),
        PlaybackClientError::CoverArtError(msg) => (
            format!("Could not process cover: {}", msg),
            button(Text::new("Dismiss")).on_press(Message::DismissError),
        ),
        PlaybackClientError::InvalidAlbumIndexError => (
            String::from("Invalid album index!"),
            button(Text::new("Dismiss")).on_press(Message::DismissError),
        ),
    };
    Container::new(
        Column::new()
            .align_x(Alignment::Center)
            .push(Text::new("Error"))
            .push(Text::new(error_text))
            .push(dismiss_or_exit_button),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .default_font(UI_FONT)
        .run()
}