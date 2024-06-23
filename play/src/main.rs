use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use libmpv::{
    events::{Event, PropertyData},
    mpv_end_file_reason, Mpv,
};
use rand::prelude::SliceRandom;
use ratatui::{
    backend::CrosstermBackend,
    layout::Layout,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use serde::Deserialize;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::{
    io::stdout,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

enum InternalEvent {
    Pos(f64),
    Duration(f64),
    Eof,
}

enum InternalControl {
    Play,
    Pause,
    Seek(f64),
    Open(PathBuf),
    Quit,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Metadata {
    title: String,
    title_unicode: String,
    artist: String,
    artist_unicode: String,
    source: String,
    #[allow(dead_code)]
    tags: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JsonItem {
    audio_hash: String,
    #[serde(rename = "BGHash")]
    bg_hash: String,
    metadata: Metadata,
}

fn get_file_path(osu_path: &Path, hash: &str) -> PathBuf {
    osu_path.join(&hash[0..1]).join(&hash[0..2]).join(hash)
}

struct App {
    progress: f64,
    total: f64,
    paused: bool,
    idx: usize,
    title: String,
    artist: String,
    source: String,
    is_unicode: bool,
    bg_img: Box<dyn StatefulProtocol>,
    osu_path: PathBuf,
    json_item: Vec<JsonItem>,
    controls: MediaControls,
}

impl App {
    fn new(
        mut picker: ratatui_image::picker::Picker,
        controls: MediaControls,
        osu_path: &Path,
        json_item: Vec<JsonItem>,
    ) -> Self {
        App {
            progress: 0.0,
            total: 0.0,
            paused: false,
            idx: 0,
            title: String::new(),
            artist: String::new(),
            source: String::new(),
            is_unicode: false,
            bg_img: picker.new_resize_protocol(image::DynamicImage::new_rgb8(1, 1)),
            osu_path: osu_path.to_path_buf(),
            json_item,
            controls,
        }
    }

    fn open(&self, mpv_control_tx: mpsc::Sender<InternalControl>) {
        mpv_control_tx
            .send(InternalControl::Open(get_file_path(
                &self.osu_path,
                &self.json_item[self.idx].audio_hash,
            )))
            .unwrap();
    }

    fn update_metadata(&mut self, mut picker: Option<ratatui_image::picker::Picker>) {
        let item = &self.json_item[self.idx];
        self.title = if !self.is_unicode {
            item.metadata.title.clone()
        } else {
            let u = item.metadata.title_unicode.clone();
            if u.trim().is_empty() {
                item.metadata.title.clone()
            } else {
                u
            }
        };
        self.artist = if !self.is_unicode {
            item.metadata.artist.clone()
        } else {
            let u = item.metadata.artist_unicode.clone();
            if u.trim().is_empty() {
                item.metadata.artist.clone()
            } else {
                u
            }
        };
        self.source.clone_from(&item.metadata.source);
        self.set_metadata();

        if let Some(picker) = picker.as_mut() {
            self.bg_img = picker.new_resize_protocol(
                image::io::Reader::open(get_file_path(
                    &self.osu_path,
                    &self.json_item[self.idx].bg_hash,
                ))
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap(),
            );
        }
    }

    fn update_progress(&mut self, progress: f64) {
        self.progress = progress;
        self.set_playback();
    }

    fn set_playback(&mut self) {
        if !self.paused {
            self.controls
                .set_playback(souvlaki::MediaPlayback::Playing {
                    progress: Some(souvlaki::MediaPosition(Duration::from_secs_f64(
                        self.progress,
                    ))),
                })
                .unwrap();
        } else {
            self.controls
                .set_playback(souvlaki::MediaPlayback::Paused {
                    progress: Some(souvlaki::MediaPosition(Duration::from_secs_f64(
                        self.progress,
                    ))),
                })
                .unwrap();
        }
    }

    fn update_duration(&mut self, total: f64) {
        self.total = total;
        self.set_metadata();
    }

    fn set_metadata(&mut self) {
        self.controls
            .set_metadata(MediaMetadata {
                title: Some(&self.title),
                artist: Some(&self.artist),
                album: Some(&self.source),
                duration: Some(Duration::from_secs_f64(self.total)),
                ..Default::default()
            })
            .unwrap();
    }

    fn next_idx(&mut self) {
        self.idx += 1;
        if self.idx >= self.json_item.len() {
            self.idx = 0;
        }
    }

    fn prev_idx(&mut self) {
        if self.idx == 0 {
            self.idx = self.json_item.len() - 1;
        } else {
            self.idx -= 1;
        }
    }

    fn toggle_unicode(&mut self) {
        self.is_unicode = !self.is_unicode;
        self.update_metadata(None);
    }

    fn set_paused(&mut self, paused: bool, mpv_control_tx: mpsc::Sender<InternalControl>) {
        self.paused = paused;
        if self.paused {
            mpv_control_tx.send(InternalControl::Pause).unwrap();
        } else {
            mpv_control_tx.send(InternalControl::Play).unwrap();
        }
        self.set_playback();
    }
}

fn main() {
    let json_file = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let osu_path = PathBuf::from(std::env::args().nth(2).unwrap());
    let mut json_item: Vec<JsonItem> = serde_json::from_str(&json_file).unwrap();
    json_item.shuffle(&mut rand::thread_rng());

    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    terminal.clear().unwrap();
    let mut picker = Picker::from_termios().unwrap_or(Picker::new((7, 14)));
    picker.guess_protocol();

    let mpv = Mpv::with_initializer(|c| c.set_property("load-scripts", "no")).unwrap();
    mpv.set_property("vo", "null").unwrap();
    mpv.set_property("volume", 100).unwrap();

    let souvlaki_config = PlatformConfig {
        dbus_name: "moe.taoky.osu-player-tools",
        display_name: "osu-player-tools",
        hwnd: None,
    };
    let mut controls = MediaControls::new(souvlaki_config).unwrap();

    let (souvlaki_tx, souvlaki_rx) = mpsc::sync_channel(32);
    controls
        .attach(move |e| souvlaki_tx.send(e).unwrap())
        .unwrap();
    controls
        .set_playback(souvlaki::MediaPlayback::Paused { progress: None })
        .unwrap();

    let (mpv_control_tx, mpv_control_rx) = mpsc::channel();
    let (mpv_event_tx, mpv_event_rx) = mpsc::channel();

    let _handle = std::thread::Builder::new()
        .name("mpv event loop".to_string())
        .spawn(move || {
            let mut ev_ctx = mpv.create_event_context();
            ev_ctx.disable_deprecated_events().unwrap();
            ev_ctx
                .observe_property("time-pos", libmpv::Format::Double, 0)
                .unwrap();
            ev_ctx
                .observe_property("duration", libmpv::Format::Double, 1)
                .unwrap();
            loop {
                let event = ev_ctx.wait_event(0.16).unwrap_or(Err(libmpv::Error::Null));
                match event {
                    Ok(Event::StartFile) => {}
                    Ok(Event::EndFile(r)) => {
                        if r == mpv_end_file_reason::Eof {
                            mpv_event_tx.send(InternalEvent::Eof).unwrap();
                        }
                    }
                    Ok(Event::PropertyChange {
                        name,
                        change,
                        reply_userdata: _,
                    }) => match name {
                        "time-pos" => {
                            if let PropertyData::Double(time) = change {
                                mpv_event_tx
                                    .send(InternalEvent::Pos(time.max(0.0)))
                                    .unwrap();
                            }
                        }
                        "duration" => {
                            if let PropertyData::Double(duration) = change {
                                mpv_event_tx
                                    .send(InternalEvent::Duration(duration.max(0.0)))
                                    .unwrap();
                            }
                        }
                        _ => {}
                    },
                    Ok(_) => {}
                    Err(e) => {
                        if e != libmpv::Error::Null {
                            println!("Error: {:?}", e);
                        }
                    }
                }

                if let Ok(control) = mpv_control_rx.try_recv() {
                    match control {
                        InternalControl::Play => {
                            mpv.set_property("pause", false).unwrap();
                        }
                        InternalControl::Pause => {
                            mpv.set_property("pause", true).unwrap();
                        }
                        InternalControl::Seek(time) => {
                            mpv.set_property("time-pos", time).unwrap();
                        }
                        InternalControl::Open(path) => {
                            mpv.command("loadfile", &[path.to_str().unwrap(), "replace"])
                                .unwrap();
                        }
                        InternalControl::Quit => {
                            mpv.command("quit", &[]).unwrap();
                            break;
                        }
                    }
                }
            }
        })
        .unwrap();

    let mut app = App::new(picker, controls, &osu_path, json_item);

    app.open(mpv_control_tx.clone());
    app.update_metadata(Some(picker));

    loop {
        if let Ok(msg) = mpv_event_rx.try_recv() {
            match msg {
                InternalEvent::Pos(time) => {
                    app.update_progress(time);
                }
                InternalEvent::Eof => {
                    app.next_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                InternalEvent::Duration(duration) => {
                    app.update_duration(duration);
                }
            }
        }
        terminal
            .draw(|frame| {
                let outer_block = Block::default()
                    .title("osu! player tools")
                    .borders(Borders::TOP);
                let chunks = Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            ratatui::layout::Constraint::Percentage(10),
                            ratatui::layout::Constraint::Percentage(90),
                        ]
                        .as_ref(),
                    )
                    .split(outer_block.inner(frame.size()));
                frame.render_widget(outer_block, frame.size());
                frame.render_widget(
                    Paragraph::new(format!(
                        "{} - {} {:.1} / {:.1} (paused? {})",
                        app.title, app.artist, app.progress, app.total, app.paused
                    )),
                    chunks[0],
                );
                let imgw = StatefulImage::new(None);
                frame.render_stateful_widget(imgw, chunks[1], &mut app.bg_img);
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                if key_event.code == event::KeyCode::Char('q') {
                    mpv_control_tx.send(InternalControl::Quit).unwrap();
                    break;
                }
                if key_event.code == event::KeyCode::Char(' ') {
                    app.set_paused(!app.paused, mpv_control_tx.clone());
                }
                if key_event.code == event::KeyCode::Char('>') {
                    app.next_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                if key_event.code == event::KeyCode::Char('<') {
                    app.prev_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                if key_event.code == event::KeyCode::Char('u') {
                    app.toggle_unicode();
                }
                if key_event.code == event::KeyCode::Left {
                    mpv_control_tx
                        .send(InternalControl::Seek(app.progress - 5.0))
                        .unwrap();
                }
                if key_event.code == event::KeyCode::Right {
                    mpv_control_tx
                        .send(InternalControl::Seek(app.progress + 5.0))
                        .unwrap();
                }
            }
        }

        for event in souvlaki_rx.try_iter() {
            match event {
                MediaControlEvent::Toggle => {
                    app.set_paused(!app.paused, mpv_control_tx.clone());
                }
                MediaControlEvent::Play => {
                    app.set_paused(false, mpv_control_tx.clone());
                }
                MediaControlEvent::Pause => {
                    app.set_paused(true, mpv_control_tx.clone());
                }
                MediaControlEvent::Next => {
                    app.next_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                MediaControlEvent::Previous => {
                    app.prev_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                _ => (),
            }
        }
    }

    // Well we don't need to join mpv thread: it will be killed after main() is done
    // handle.join().unwrap();
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
