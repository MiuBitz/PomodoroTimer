#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod settings;
mod startup;
mod timer;
use settings::Settings;
use std::{sync::Mutex, time::Duration};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, State, WindowEvent};
use timer::{Snapshot, Timer};
struct AppState {
    settings: Settings,
    timer: Timer,
}
fn lock(state: &Mutex<AppState>) -> Result<std::sync::MutexGuard<'_, AppState>, String> {
    state.lock().map_err(|_| "Timer state unavailable".into())
}
#[tauri::command]
fn get_state(state: State<Mutex<AppState>>) -> Result<Snapshot, String> {
    let state = lock(&state)?;
    Ok(state
        .timer
        .snapshot(&state.settings, std::time::Instant::now()))
}
#[tauri::command]
fn get_settings(state: State<Mutex<AppState>>) -> Result<Settings, String> {
    Ok(lock(&state)?.settings.clone())
}
#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    state: State<Mutex<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    settings.validate()?;
    let mut state = lock(&state)?;
    let changed = settings.start_with_windows != state.settings.start_with_windows
        || settings.start_with_windows;
    if changed {
        startup::set_startup(settings.start_with_windows)?;
    }
    if let Err(e) = settings.save() {
        if changed {
            let _ = startup::set_startup(state.settings.start_with_windows);
        }
        return Err(e);
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_theme(Some(if settings.theme_mode == "light" {
            tauri::Theme::Light
        } else {
            tauri::Theme::Dark
        }));
    }
    state.settings = settings;
    Ok(())
}
#[tauri::command]
fn timer_action(
    state: State<Mutex<AppState>>,
    action: String,
    cycles: Option<u32>,
) -> Result<Snapshot, String> {
    let mut state = lock(&state)?;
    let now = std::time::Instant::now();
    match action.as_str() {
        "start" => {
            if state.timer.running {
                return Err("A session is already running".into());
            }
            let mut config = state.settings.clone();
            config.cycles = cycles.unwrap_or(config.cycles);
            config.validate()?;
            config.save()?;
            state.timer.start(&config, now);
            state.settings = config;
        }
        "pause" => state.timer.pause(now),
        "resume" => state.timer.resume(now),
        "reset" => state.timer = Timer::default(),
        _ => return Err("Unknown timer action".into()),
    }
    Ok(state.timer.snapshot(&state.settings, now))
}
fn show(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
fn main() {
    let settings = Settings::load();
    let hidden =
        std::env::args().any(|a| a == "--startup" || a == "--autostart" || a == "--minimized");
    tauri::Builder::default()
        .manage(Mutex::new(AppState {
            settings,
            timer: Timer::default(),
        }))
        .setup(move |app| {
            let open = MenuItem::with_id(app, "open", "Open Pomodoro", true, None::<&str>)?;
            let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &exit])?;
            TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .ok_or("Application icon missing")?
                        .clone(),
                )
                .tooltip("Pomodoro Timer")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "open" => show(app),
                    "exit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if matches!(
                        e,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        show(tray.app_handle());
                    }
                })
                .build(app)?;
            let config = lock(app.state::<Mutex<AppState>>().inner())?
                .settings
                .clone();
            if config.start_with_windows {
                if let Err(error) = startup::set_startup(true) {
                    eprintln!("Could not update startup registration: {error}");
                }
            }
            if let Some(w) = app.get_webview_window("main") {
                w.set_theme(Some(if config.theme_mode == "light" {
                    tauri::Theme::Light
                } else {
                    tauri::Theme::Dark
                }))?;
            }
            if !hidden {
                show(app.handle());
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(200));
                let state = handle.state::<Mutex<AppState>>();
                let notification = if let Ok(mut state) = state.lock() {
                    let message = state.timer.advance(std::time::Instant::now());
                    message.map(|m| {
                        (
                            m,
                            state.settings.notifications_enabled,
                            state.settings.sound_enabled,
                        )
                    })
                } else {
                    None
                };
                if let Some((message, notify, sound)) = notification {
                    if notify {
                        // The default Windows host works for portable executables too.
                        // Toast audio is silent; the separate sound toggle owns the beep.
                        if let Err(e) = notify_rust::Notification::new()
                            .summary("Pomodoro Timer")
                            .body(message)
                            .show()
                        {
                            eprintln!("Notification: {e}");
                        }
                    }
                    if sound {
                        unsafe {
                            MessageBeep(0x40);
                        }
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Mutex<AppState>>();
                if let Ok(state) = state.lock() {
                    if state.settings.minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                };
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_settings,
            save_settings,
            timer_action
        ])
        .run(tauri::generate_context!())
        .expect("Unable to run Pomodoro Timer");
}
#[link(name = "user32")]
extern "system" {
    fn MessageBeep(kind: u32) -> i32;
}
