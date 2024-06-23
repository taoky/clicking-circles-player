use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use libmpv::{
    events::{Event, PropertyData},
    Mpv,
};
use ratatui::{backend::CrosstermBackend, style::Stylize, widgets::Paragraph, Terminal};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::{io::stdout, sync::mpsc, time::Duration};

enum InternalEvent {
    Pos(f64),
    Duration(f64),
    Eof,
}

enum InternalControl {
    Play,
    Pause,
    Seek(f64),
}

fn main() {
    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    terminal.clear().unwrap();

    let mpv = Mpv::new().unwrap();
    mpv.set_property("vo", "null").unwrap();
    mpv.set_property("volume", 100).unwrap();

    let souvlaki_config = PlatformConfig {
        dbus_name: "moe.taoky.osu-player-tools",
        display_name: "osu-player-tools",
        hwnd: None,
    };
    let mut controls = MediaControls::new(souvlaki_config).unwrap();

    let (souvlaki_tx, souvlaki_rx) = mpsc::channel();
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
            // mpv_control_rx and mpv_event_tx in the thread
            mpv.command(
                "loadfile",
                &[r#"https://www.youtube.com/watch?v=dQw4w9WgXcQ"#, "replace"],
            )
            .unwrap();
            // mpv.set_property("pause", false).unwrap();
            let mut ev_ctx = mpv.create_event_context();
            ev_ctx.disable_deprecated_events().unwrap();
            ev_ctx
                .observe_property("time-pos", libmpv::Format::Double, 0)
                .unwrap();
            ev_ctx
                .observe_property("duration", libmpv::Format::Double, 1)
                .unwrap();
            loop {
                let event = ev_ctx.wait_event(0.0).unwrap_or(Err(libmpv::Error::Null));
                match event {
                    Ok(Event::StartFile) => {}
                    Ok(Event::EndFile(_)) => {
                        mpv_event_tx.send(InternalEvent::Eof).unwrap();
                    }
                    Ok(Event::PropertyChange {
                        name,
                        change,
                        reply_userdata: _,
                    }) => match name {
                        "time-pos" => {
                            if let PropertyData::Double(time) = change {
                                mpv_event_tx.send(InternalEvent::Pos(time)).unwrap();
                            }
                        }
                        "duration" => {
                            if let PropertyData::Double(duration) = change {
                                mpv_event_tx
                                    .send(InternalEvent::Duration(duration))
                                    .unwrap();
                            }
                        }
                        _ => {}
                    },
                    Ok(_) => {}
                    Err(_) => {}
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
                    }
                }
            }
        })
        .unwrap();

    let mut progress = 0.0;
    let mut total = 0.0;
    let mut end = false;
    let mut paused = true;
    loop {
        if let Ok(msg) = mpv_event_rx.try_recv() {
            match msg {
                InternalEvent::Pos(time) => {
                    progress = time;
                }
                InternalEvent::Eof => {
                    end = true;
                }
                InternalEvent::Duration(duration) => {
                    total = duration;
                    controls
                        .set_metadata(MediaMetadata {
                            title: Some("Never gonna give you up"),
                            artist: Some("Rick Astley"),
                            album: Some("Whenever You Need Somebody"),
                            duration: Some(Duration::from_secs_f64(duration)),
                            ..Default::default()
                        })
                        .unwrap();
                }
            }
        }
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(
                    Paragraph::new(format!(
                        "Never gonna give you up {}/{}/{}",
                        progress, total, end
                    ))
                    .white()
                    .on_blue(),
                    area,
                );
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                if key_event.code == event::KeyCode::Char('q') {
                    break;
                }
                if key_event.code == event::KeyCode::Char(' ') {
                    if paused {
                        mpv_control_tx.send(InternalControl::Play).unwrap();
                        controls
                            .set_playback(souvlaki::MediaPlayback::Playing {
                                progress: Some(souvlaki::MediaPosition(Duration::from_secs_f64(
                                    progress,
                                ))),
                            })
                            .unwrap();
                    } else {
                        mpv_control_tx.send(InternalControl::Pause).unwrap();
                        controls
                            .set_playback(souvlaki::MediaPlayback::Paused {
                                progress: Some(souvlaki::MediaPosition(Duration::from_secs_f64(
                                    progress,
                                ))),
                            })
                            .unwrap();
                    }
                    paused = !paused;
                }
            }
        }

        for event in souvlaki_rx.try_iter() {
            match event {
                MediaControlEvent::Toggle => {

                }
                MediaControlEvent::Play => {}
                MediaControlEvent::Pause => {}
                _ => (),
            }
        }
    }

    // Well we don't need to join mpv thread: it will be killed after main() is done
    // handle.join().unwrap();
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
