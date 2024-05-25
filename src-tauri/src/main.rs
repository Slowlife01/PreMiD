#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{env::current_exe, path::Path};
use tauri::{
    generate_handler, AppHandle, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

use axum::Server;
use socketioxide::{adapter::LocalAdapter, Namespace, Socket, SocketIoLayer};
// use tower_http::validate_request::ValidateRequestHeaderLayer;

use notify::Watcher;

use crossbeam_channel::unbounded;
use discord_presence::{
    models::{Activity, ActivityButton},
    Client,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri_api::dialog;

use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    id: String,
    username: String,
    avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SetActivityData {
    #[serde(rename(deserialize = "clientId"))]
    client_id: String,
    #[serde(rename(deserialize = "presenceData"))]
    data: PresenceData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PresenceData {
    details: Option<String>,
    state: Option<String>,
    #[serde(rename(deserialize = "largeImageText"))]
    large_image_text: Option<String>,
    #[serde(rename(deserialize = "largeImageKey"))]
    large_image_key: Option<String>,
    #[serde(rename(deserialize = "smallImageText"))]
    small_image_text: Option<String>,
    #[serde(rename(deserialize = "smallImageKey"))]
    small_image_key: Option<String>,
    #[serde(rename(deserialize = "startTimestamp"))]
    start_timestamp: Option<u64>,
    #[serde(rename(deserialize = "endTimestamp"))]
    end_timestamp: Option<u64>,
    buttons: Option<Vec<ActivityButton>>,
}

#[derive(Serialize, Debug)]
struct AppState {
    connected: bool,
    user: Option<User>,
}

#[tauri::command]
fn get_user(handle: AppHandle) -> Result<User, ()> {
    let app_state = handle.try_state::<Arc<Mutex<Option<AppState>>>>();
    let socket = handle.try_state::<Mutex<Arc<Socket<LocalAdapter>>>>();

    if let Some(app_state) = app_state {
        let lock = app_state.lock().unwrap();
        let app_state = lock.deref().as_ref();

        if let Some(app_state) = app_state {
            return Ok(app_state.user.clone().unwrap());
        } else {
            drop(lock);

            let socket = socket.clone();
            if let Some(socket) = socket {
                let mut client = Client::new(503557087041683458);
                _ = client.start();

                client.on_ready({
                    let handle = handle.clone();
                    move |ctx| {
                        let user =
                            serde_json::from_value::<User>(ctx.event["user"].clone()).unwrap();

                        let app_state = handle.state::<Arc<Mutex<Option<AppState>>>>();
                        let mut lock = app_state.lock().unwrap();

                        *lock = Some(AppState {
                            connected: true,
                            user: Some(user),
                        });
                    }
                });

                std::thread::sleep(std::time::Duration::from_millis(500));
                client.clear();

                if let Some(app_state) = handle.try_state::<Arc<Mutex<Option<AppState>>>>() {
                    let socket = socket.lock().unwrap();

                    let app_state = app_state.lock().unwrap();
                    let app_state = app_state.deref().as_ref();

                    if let Some(app_state) = app_state {
                        socket.emit("discordUser", app_state.user.clone()).unwrap();
                        return Ok(app_state.user.clone().unwrap());
                    }
                }
            }
        }
    }

    Err(())
}

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            SystemTray::new()
                .with_tooltip("PreMiD")
                .with_menu(
                    SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
                )
                .on_event(move |event| match event {
                    SystemTrayEvent::LeftClick { .. } => {
                        let window = handle.get_window("main");
                        match window {
                            Some(window) => {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                            None => {}
                        };
                    }
                    SystemTrayEvent::MenuItemClick { id, .. } => {
                        if id == "quit" {
                            handle.exit(0);
                        }
                    }
                    _ => (),
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|e| match e.event() {
            WindowEvent::CloseRequested { api, .. } => {
                e.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .manage(Arc::new(Mutex::new(None::<AppState>)))
        .invoke_handler(generate_handler![get_user])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    let (tx2, rx2) = unbounded::<SetActivityData>();
    let (tx3, rx3) = unbounded::<()>();
    let (tx4, rx4) = unbounded::<String>();

    let handle = Arc::new(app.handle());

    std::thread::spawn({
        let handle = handle.clone();
        let tx2 = tx2.clone();

        move || {
            let client = Arc::new(Mutex::new(None::<Client>));
            let watcher = notify::recommended_watcher({
                let handle = handle.clone();

                move |res| match res {
                    Ok(_) => {
                        let socket = handle.try_state::<Mutex<Arc<Socket<LocalAdapter>>>>();
                        if let Some(socket) = socket {
                            let socket = socket.lock().unwrap();
                            // TODO: Load local presence
                        }
                    }
                    Err(_) => {}
                }
            })
            .unwrap();

            let watcher = Arc::new(Mutex::new(watcher));
            let prev_path = Arc::new(Mutex::new(None::<String>));

            loop {
                if let Ok(path) = rx4.try_recv() {
                    let mut watcher = watcher.lock().unwrap();
                    let mut prev_path_lock = prev_path.lock().unwrap();

                    if prev_path_lock.is_some() {
                        let prev_path = prev_path_lock.as_ref();
                        let prev_path = prev_path.unwrap().as_str();
                        watcher.unwatch(Path::new(prev_path)).ok();

                        let path = path.clone();
                        *prev_path_lock = Some(path);
                    };

                    watcher
                        .watch(Path::new(path.as_str()), notify::RecursiveMode::Recursive)
                        .ok();
                }

                if let Ok(_) = rx3.try_recv() {
                    let mut lock = client.lock().unwrap();
                    if lock.is_some() {
                        let client = lock.as_mut().unwrap();
                        client.clear_activity().ok();
                    }
                }

                if let Ok(activity) = rx2.try_recv() {
                    let mut lock = client.lock().unwrap();

                    if lock.is_none() {
                        let mut client = Client::new(activity.client_id.parse().unwrap());
                        _ = client.start();

                        *lock = Some(client);
                    } else if lock.as_ref().unwrap().client_id()
                        != activity.client_id.parse::<u64>().unwrap()
                    {
                        let client = lock.as_mut().unwrap();
                        client.clear();

                        let mut client = Client::new(activity.client_id.parse().unwrap());
                        _ = client.start();

                        *lock = Some(client);
                    }

                    let client = lock.as_mut().unwrap();

                    let data = activity.data.clone();
                    let activity_data = Activity::new()
                        .details(data.details)
                        .state(data.state)
                        .timestamps(|t| t.start(data.start_timestamp).end(data.end_timestamp))
                        .assets(|a| {
                            a.large_image(data.large_image_key)
                                .large_text(Option("It's Rust.. kinda".to_string()))
                                .small_image(data.small_image_key)
                                .small_text(data.small_image_text)
                        })
                        .buttons(data.buttons);

                    if let Err(_) = client.set_activity(|_| activity_data) {
                        _ = client.start();
                        tx2.send(activity).ok();

                        let state = handle.try_state::<Arc<Mutex<Option<AppState>>>>();

                        if let Some(state) = state {
                            let mut state = state.lock().unwrap();
                            *state = None;
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    });

    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let handle = handle.clone();
        let ns = Namespace::builder()
            .add("/", move |socket| {
                if !handle.manage(Mutex::new(socket.clone())) {
                    let state = handle.state::<Mutex<Arc<Socket<LocalAdapter>>>>();
                    let mut state = state.lock().unwrap();
                    *state = socket.clone();
                };

                let app_state = handle.try_state::<Arc<Mutex<Option<AppState>>>>();
                if let Some(app_state) = app_state {
                    let app_state = app_state.lock().unwrap();
                    let state = app_state.as_ref();

                    if let Some(state) = state {
                        socket.emit("discordUser", state.user.clone()).unwrap();
                    }
                }

                let tx2 = tx2.clone();
                let tx3 = tx3.clone();
                let tx4 = tx4.clone();

                async move {
                    socket.on("selectLocalPresence", move |_, _data: Value, _, _| {
                        let tx4 = tx4.clone();
                        async move {
                            let path = pick_folder();
                            if let Ok(path) = path {
                                tx4.send(path).unwrap();
                            }
                        }
                    });

                    socket.on("clearActivity", move |_, _data: Value, _, _| {
                        tx3.send(()).unwrap();
                        async {}
                    });

                    socket.on("setActivity", move |_, data: SetActivityData, _, _| {
                        tx2.send(data).unwrap();
                        async {}
                    });
                }
            })
            .build();

        let app = axum::Router::new().layer(SocketIoLayer::new(ns));
        // .layer(
        //    Alpha sends an origin header (chrome-extension::*)
        //
        //     ValidateRequestHeaderLayer::custom(|request: &mut Request<Body>| {
        //         if request.headers().contains_key("origin") {
        //             Err(StatusCode::BAD_REQUEST.into_response())
        //         } else {
        //             Ok(())
        //         }
        //     }),
        // );

        Server::bind(&"127.0.0.1:3020".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    app.run(|_, _| {})
}

fn pick_folder() -> Result<String, ()> {
    if let Ok(response) = dialog::pick_folder(None::<&Path>) {
        return match response {
            dialog::Response::Okay(path) => Ok(path),
            dialog::Response::OkayMultiple(_) => Err(()),
            dialog::Response::Cancel => Err(()),
        };
    }

    Err(())
}
