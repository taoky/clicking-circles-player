use clap::Parser;
use crossterm::{
    event::{self},
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
    },
};
use image::{DynamicImage, GenericImageView, imageops::crop_imm};
use keepawake::KeepAwake;
use libmpv::{
    Mpv,
    events::{Event, PropertyData},
    mpv_end_file_reason,
};
use rand::prelude::SliceRandom;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Layout,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, block::Title},
};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};
use serde::{Deserialize, Deserializer};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::{
    io::{self, stdout},
    panic::{set_hook, take_hook},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};
use tui_input::backend::crossterm::EventHandler;
use url::Url;

const APP_ID: &str = "moe.taoky.clicking-circles-player";
const APP_NAME: &str = "clicking-circles-player";

enum InternalEvent {
    Pos(f64),
    Duration(f64),
    Eof,
    Quit,
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
    #[serde(deserialize_with = "vec_to_space_joined")]
    tags: String,
}

fn vec_to_space_joined<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;

    Ok(vec.join(" "))
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JsonItem {
    audio_hash: String,
    #[serde(rename = "BGHashes")]
    bg_hashes: Vec<String>,
    #[serde(flatten)]
    metadata: Metadata,
}

fn get_file_path(osu_path: &Path, hash: &str) -> PathBuf {
    osu_path.join(&hash[0..1]).join(&hash[0..2]).join(hash)
}

fn center_largest_square_crop<I: GenericImageView>(img: &I) -> image::SubImage<&I> {
    let (w, h) = img.dimensions();
    let side_len = w.min(h);
    let x = (w - side_len) / 2;
    let y = (h - side_len) / 2;
    crop_imm(img, x, y, side_len, side_len)
}

#[derive(Debug, PartialEq, Eq)]
enum UIState {
    Main,
    Search,
}

#[derive(Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    Editing,
}

struct SearchState {
    input: tui_input::Input,
    input_mode: InputMode,
    results: Vec<usize>,
    list_state: ListState,
    list_height: u16,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            input: tui_input::Input::default(),
            input_mode: InputMode::Editing,
            results: Vec::new(),
            list_state: ListState::default(),
            list_height: 1,
        }
    }
}

fn build_awake() -> Result<KeepAwake, keepawake::Error> {
    keepawake::Builder::default()
        .display(false)
        .idle(false)
        .sleep(true)
        .app_reverse_domain(APP_ID)
        .app_name(APP_NAME)
        .reason("Playing music")
        .create()
}

fn build_awake_anyway() -> Option<KeepAwake> {
    build_awake().ok()
}

fn set_terminal_title(title: &str) {
    let _ = crossterm::execute!(io::stdout(), SetTitle(title));
}

struct App {
    progress: f64,
    total: f64,
    paused: bool,
    idx: usize,
    title: String,
    artist: String,
    source: String,
    cover_path: Option<PathBuf>,
    is_unicode: bool,
    bg_img: Box<dyn StatefulProtocol>,
    osu_path: PathBuf,
    json_item: Vec<JsonItem>,
    controls: MediaControls,
    xdg_dirs: xdg::BaseDirectories,
    ui_state: UIState,
    search_state: SearchState,
    repeat: bool,
    awake: Option<KeepAwake>,
    ui_dirty: bool,
}

macro_rules! get_current_item {
    ($self: expr) => {
        $self.json_item[$self.idx]
    };
}

fn empty_image() -> DynamicImage {
    image::DynamicImage::new_rgb8(0, 0)
}

impl App {
    fn new(
        mut picker: ratatui_image::picker::Picker,
        controls: MediaControls,
        osu_path: &Path,
        json_item: Vec<JsonItem>,
        xdg_dirs: xdg::BaseDirectories,
    ) -> Self {
        App {
            progress: 0.0,
            total: 0.0,
            paused: false,
            idx: 0,
            title: String::new(),
            artist: String::new(),
            source: String::new(),
            cover_path: None,
            is_unicode: false,
            bg_img: picker.new_resize_protocol(empty_image()),
            osu_path: osu_path.to_path_buf(),
            json_item,
            controls,
            xdg_dirs,
            ui_state: UIState::Main,
            search_state: SearchState::default(),
            repeat: false,
            awake: build_awake_anyway(),
            ui_dirty: true,
        }
    }

    fn open(&self, mpv_control_tx: mpsc::Sender<InternalControl>) {
        mpv_control_tx
            .send(InternalControl::Open(get_file_path(
                &self.osu_path,
                &get_current_item!(self).audio_hash,
            )))
            .unwrap();
    }

    fn get_title(&self, item: &JsonItem) -> String {
        if !self.is_unicode {
            item.metadata.title.clone()
        } else {
            let u = item.metadata.title_unicode.clone();
            if u.trim().is_empty() {
                item.metadata.title.clone()
            } else {
                u
            }
        }
    }

    fn get_artist(&self, item: &JsonItem) -> String {
        if !self.is_unicode {
            item.metadata.artist.clone()
        } else {
            let u = item.metadata.artist_unicode.clone();
            if u.trim().is_empty() {
                item.metadata.artist.clone()
            } else {
                u
            }
        }
    }

    fn construct_terminal_title(&self) -> String {
        format!(
            "{} - {} {}",
            APP_NAME,
            if self.paused { "Paused" } else { "Playing" },
            self.get_title(&get_current_item!(self))
        )
    }

    fn update_metadata(&mut self, mut picker: Option<ratatui_image::picker::Picker>) {
        let item = &get_current_item!(self);
        self.title = self.get_title(item);
        self.artist = self.get_artist(item);
        self.source.clone_from(&item.metadata.source);

        if let Some(picker) = picker.as_mut() {
            let bg_hashes = &get_current_item!(self).bg_hashes;
            // randomly choose one
            let bg_hash = bg_hashes.choose(&mut rand::thread_rng());
            match bg_hash {
                Some(bg_hash) => {
                    let image = image::ImageReader::open(get_file_path(&self.osu_path, bg_hash))
                        .unwrap()
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                        .unwrap()
                        .to_rgb8();
                    // check if we shall generate a cover...
                    let cache_filename = format!("{}.cover.jpg", bg_hash);
                    let cache_path = self.xdg_dirs.place_cache_file(cache_filename).unwrap();
                    if !cache_path.exists() {
                        let cover = center_largest_square_crop(&image);
                        cover
                            .to_image()
                            .save_with_format(cache_path.clone(), image::ImageFormat::Jpeg)
                            .unwrap();
                    }
                    self.cover_path = Some(cache_path);

                    self.bg_img = picker.new_resize_protocol(image::DynamicImage::ImageRgb8(image));
                }
                None => {
                    self.cover_path = None;
                    self.bg_img = picker.new_resize_protocol(empty_image())
                }
            };
        }
        self.set_metadata();
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
        set_terminal_title(&self.construct_terminal_title());
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
                cover_url: self
                    .cover_path
                    .as_ref()
                    .map(|p| Url::from_file_path(p).unwrap().to_string())
                    .as_deref(),
            })
            .expect("Cannot set metadata, is there another instance running?");
        set_terminal_title(&self.construct_terminal_title());
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
            self.awake = None;
            mpv_control_tx.send(InternalControl::Pause).unwrap();
        } else {
            if self.awake.is_none() {
                self.awake = build_awake_anyway();
            }
            mpv_control_tx.send(InternalControl::Play).unwrap();
        }
        self.set_playback();
    }

    fn search(&self, query: &str) -> Vec<usize> {
        let mut result = Vec::new();
        let query = query.to_lowercase();
        for (i, item) in self.json_item.iter().enumerate() {
            if item.metadata.title.to_ascii_lowercase().contains(&query)
                || item.metadata.artist.to_ascii_lowercase().contains(&query)
                || item.metadata.source.to_lowercase().contains(&query)
                || item.metadata.title_unicode.to_lowercase().contains(&query)
                || item.metadata.artist_unicode.to_lowercase().contains(&query)
                || item.metadata.tags.to_lowercase().contains(&query)
            {
                result.push(i);
            }
        }
        result
    }

    fn item_to_string(&self, i: usize) -> String {
        let item = &self.json_item[i];
        format!("{} - {}", self.get_title(item), self.get_artist(item))
    }
}

fn main_ui<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mpv_control_tx: mpsc::Sender<InternalControl>,
    picker: Picker,
) where
    B: ratatui::backend::Backend,
{
    if app.ui_dirty {
        terminal
            .draw(|frame| {
                let outer_block = Block::default()
                    .title("clicking circles player")
                    .title(
                        Title::from(format!("{}/{}", app.idx + 1, app.json_item.len()))
                            .alignment(ratatui::layout::Alignment::Right),
                    )
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
                        "{} - {} {:.1} / {:.1} ({}{})",
                        app.title,
                        app.artist,
                        app.progress,
                        app.total,
                        if app.paused { "paused" } else { "playing" },
                        if app.repeat { " repeat" } else { "" }
                    ))
                    .wrap(Wrap { trim: true }),
                    chunks[0],
                );
                let imgw = StatefulImage::new(None);
                frame.render_stateful_widget(imgw, chunks[1], &mut app.bg_img);
            })
            .unwrap();
        app.ui_dirty = false;
    }
    if event::poll(std::time::Duration::from_millis(16)).unwrap() {
        let tm_event = event::read().unwrap();
        if let event::Event::Key(key_event) = tm_event {
            app.ui_dirty = true;
            match key_event.code {
                event::KeyCode::Char('q') => {
                    mpv_control_tx.send(InternalControl::Quit).unwrap();
                }
                event::KeyCode::Char(' ') => {
                    app.set_paused(!app.paused, mpv_control_tx.clone());
                }
                event::KeyCode::Char('>') => {
                    app.next_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                event::KeyCode::Char('<') => {
                    app.prev_idx();
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                event::KeyCode::Char('u') => {
                    app.toggle_unicode();
                }
                event::KeyCode::Char('r') => {
                    app.repeat = !app.repeat;
                }
                event::KeyCode::Left => {
                    mpv_control_tx
                        .send(InternalControl::Seek(app.progress - 5.0))
                        .unwrap();
                }
                event::KeyCode::Right => {
                    mpv_control_tx
                        .send(InternalControl::Seek(app.progress + 5.0))
                        .unwrap();
                }
                event::KeyCode::Char('s') => {
                    app.ui_state = UIState::Search;
                }
                _ => {}
            }
        } else if let event::Event::Resize(_, _) = tm_event {
            app.ui_dirty = true;
        }
    }
}

fn search_ui<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mpv_control_tx: mpsc::Sender<InternalControl>,
    picker: Picker,
) where
    B: ratatui::backend::Backend,
{
    if app.ui_dirty {
        terminal
            .draw(|frame| {
                let outer_block = Block::default().title("searching...").borders(Borders::TOP);
                let chunks = Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            ratatui::layout::Constraint::Length(3),
                            ratatui::layout::Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(outer_block.inner(frame.size()));
                frame.render_widget(outer_block, frame.size());
                let width = chunks[0].width.max(3) - 3;
                let scroll = app.search_state.input.visual_scroll(width as usize);
                let input = Paragraph::new(app.search_state.input.value())
                    .style(match app.search_state.input_mode {
                        InputMode::Normal => Style::default(),
                        InputMode::Editing => Style::default().fg(ratatui::style::Color::Yellow),
                    })
                    .scroll((0, scroll as u16))
                    .block(Block::default().borders(Borders::ALL).title("Search"));
                frame.render_widget(input, chunks[0]);
                if app.search_state.input_mode == InputMode::Editing {
                    frame.set_cursor(
                        chunks[0].x
                            + 1
                            + (app.search_state.input.visual_cursor().max(scroll) - scroll) as u16,
                        chunks[0].y + 1,
                    );
                }
                let items: Vec<ListItem> = app
                    .search_state
                    .results
                    .iter()
                    .map(|&i| ListItem::new(app.item_to_string(i)))
                    .collect();
                let items_title = if let Some(idx) = app.search_state.list_state.selected() {
                    format!("Results ({}/{})", idx + 1, items.len())
                } else {
                    "Results".to_string()
                };
                let items = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(items_title)
                            .border_style(match app.search_state.input_mode {
                                InputMode::Normal => {
                                    Style::default().fg(ratatui::style::Color::Yellow)
                                }
                                InputMode::Editing => Style::default(),
                            }),
                    )
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::BOLD)
                            .fg(ratatui::style::Color::Yellow),
                    )
                    .highlight_symbol(">")
                    .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
                // frame.render_widget(items, chunks[1]);
                frame.render_stateful_widget(items, chunks[1], &mut app.search_state.list_state);
                app.search_state.list_height = (chunks[1].height - 2).max(1);
            })
            .unwrap();
        app.ui_dirty = false;
    }
    if event::poll(std::time::Duration::from_millis(16)).unwrap() {
        let tm_event = event::read().unwrap();
        if let event::Event::Key(key_event) = tm_event {
            app.ui_dirty = true;

            fn previous(current: usize, offset: usize) -> usize {
                current.saturating_sub(offset)
            }

            fn next(current: usize, total: usize, offset: usize) -> usize {
                current.saturating_add(offset).min(total - 1)
            }

            fn circular_previous(current: usize, total: usize, offset: usize) -> usize {
                (current + total - (offset % total)) % total
            }

            fn circular_next(current: usize, total: usize, offset: usize) -> usize {
                (current + offset) % total
            }

            match app.search_state.input_mode {
                InputMode::Normal => match key_event.code {
                    event::KeyCode::Up => {
                        let i = circular_previous(
                            app.search_state.list_state.selected().unwrap_or(0),
                            app.search_state.results.len(),
                            1,
                        );
                        app.search_state.list_state.select(Some(i));
                    }
                    event::KeyCode::Down => {
                        let i = circular_next(
                            app.search_state.list_state.selected().unwrap_or(0),
                            app.search_state.results.len(),
                            1,
                        );
                        app.search_state.list_state.select(Some(i));
                    }
                    event::KeyCode::PageUp => {
                        let i = previous(
                            app.search_state.list_state.selected().unwrap_or(0),
                            app.search_state.list_height.into(),
                        );
                        app.search_state.list_state.select(Some(i));
                    }
                    event::KeyCode::PageDown => {
                        let i = next(
                            app.search_state.list_state.selected().unwrap_or(0),
                            app.search_state.results.len(),
                            app.search_state.list_height.into(),
                        );
                        app.search_state.list_state.select(Some(i));
                    }
                    event::KeyCode::Char('u') => {
                        app.toggle_unicode();
                    }
                    event::KeyCode::Esc | event::KeyCode::Tab => {
                        app.search_state.input_mode = InputMode::Editing;
                    }
                    event::KeyCode::Enter => {
                        if let Some(i) = app.search_state.list_state.selected() {
                            app.idx = app.search_state.results[i];
                            app.open(mpv_control_tx.clone());
                            app.update_metadata(Some(picker));
                            app.set_paused(false, mpv_control_tx.clone());
                            app.ui_state = UIState::Main;
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key_event.code {
                    event::KeyCode::Esc => {
                        app.ui_state = UIState::Main;
                    }
                    event::KeyCode::Enter | event::KeyCode::Tab => {
                        app.search_state.results = app.search(app.search_state.input.value());
                        if !app.search_state.results.is_empty() {
                            app.search_state.list_state.select(Some(0));
                        } else {
                            app.search_state.list_state.select(None);
                        }
                        app.search_state.input_mode = InputMode::Normal;
                    }
                    _ => {
                        app.search_state
                            .input
                            .handle_event(&crossterm::event::Event::Key(key_event));
                    }
                },
            }
        } else if let event::Event::Resize(_, _) = tm_event {
            app.ui_dirty = true;
        } else if let event::Event::Paste(_) = tm_event {
            // search UI accepts user paste
            app.ui_dirty = true;
        }
    }
}

#[derive(Parser, Debug)]
struct Cli {
    /// Path to RealmHashExtractor's generated JSON file
    json_file: PathBuf,

    /// Path to osu! files directory
    osu_path: PathBuf,

    /// Controls loudness normalization
    #[clap(long = "loudnorm", default_value_t = true, action = clap::ArgAction::SetTrue)]
    #[clap(long = "no-loudnorm", action = clap::ArgAction::SetFalse)]
    loudnorm: bool,
}

pub fn init_tui() -> io::Result<Terminal<impl ratatui::backend::Backend>> {
    enable_raw_mode()?;
    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

fn main() {
    let args = Cli::parse();
    let json_file = std::fs::read_to_string(&args.json_file).unwrap();
    let mut json_item: Vec<JsonItem> = serde_json::from_str(&json_file).unwrap();
    json_item.shuffle(&mut rand::thread_rng());

    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME).unwrap();

    init_panic_hook();
    let mut terminal = init_tui().unwrap();
    terminal.clear().unwrap();
    let mut picker = Picker::from_termios().unwrap_or(Picker::new((7, 14)));
    picker.guess_protocol();

    let mpv = Mpv::with_initializer(|c| c.set_property("load-scripts", "no")).unwrap();
    mpv.set_property("vo", "null").unwrap();
    mpv.set_property("volume", 100).unwrap();
    if args.loudnorm {
        mpv.set_property("af", "lavfi=[loudnorm=I=-14:TP=-2:LRA=11]")
            .unwrap();
    }

    let souvlaki_config = PlatformConfig {
        dbus_name: APP_ID,
        display_name: APP_NAME,
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
                            mpv_event_tx.send(InternalEvent::Quit).unwrap();
                            break;
                        }
                    }
                }
            }
        })
        .unwrap();

    let mut app = App::new(picker, controls, &args.osu_path, json_item, xdg_dirs);

    app.open(mpv_control_tx.clone());
    app.update_metadata(Some(picker));

    loop {
        if let Ok(msg) = mpv_event_rx.try_recv() {
            app.ui_dirty = true;
            match msg {
                InternalEvent::Pos(time) => {
                    app.update_progress(time);
                }
                InternalEvent::Eof => {
                    if !app.repeat {
                        app.next_idx();
                    }
                    app.open(mpv_control_tx.clone());
                    app.update_metadata(Some(picker));
                }
                InternalEvent::Duration(duration) => {
                    app.update_duration(duration);
                }
                InternalEvent::Quit => {
                    break;
                }
            }
        }
        if app.ui_state == UIState::Main {
            main_ui(&mut terminal, &mut app, mpv_control_tx.clone(), picker);
        } else {
            search_ui(&mut terminal, &mut app, mpv_control_tx.clone(), picker);
        }

        for event in souvlaki_rx.try_iter() {
            app.ui_dirty = true;
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

    restore_tui().unwrap();
}
