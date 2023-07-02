use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

pub use minijinja;

use axum::body::Body;
use axum::extract::{ws, ConnectInfo, WebSocketUpgrade};
use axum::response::Response;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tokio::time::Instant;

use crate::{endpointof, TEMPLATES};

static PWD: Lazy<PathBuf> = Lazy::new(|| std::env::current_dir().unwrap().join("templates"));
static HMR_BROADCAST: Lazy<(broadcast::Sender<String>, broadcast::Receiver<String>)> =
    Lazy::new(|| broadcast::channel(1));

static CONNECTIONS: AtomicU8 = AtomicU8::new(0);

pub(crate) async fn hmr_websocket_handler(
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
) -> Response<Body> {
    let ip = ip.ip();
    ws.on_upgrade(move |mut socket| async move {
        let conn_id = CONNECTIONS.fetch_add(1, Ordering::SeqCst);
        TEMPLATES
            .forms
            .write()
            .unwrap()
            .insert(ip, Default::default());

        let mut rx = HMR_BROADCAST.0.subscribe();

        while let Ok(path) = rx.recv().await {
            let mut dur_start = Instant::now();
            if socket
                .send(ws::Message::Text("you up?".into()))
                .await
                .is_err()
            {
                eprintln!(
                    "(HMR) {} CONN{} (browser status)   connection closed",
                    path, conn_id
                );
                CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
                return;
            }

            if path.ends_with(".html.jinja2") {
                let endpoint = format!("/{}", endpointof(&path).unwrap());

                socket
                    .send(ws::Message::Text(endpoint.clone()))
                    .await
                    .unwrap_or_default();

                let mut escape = false;
                if let Some(Ok(ws::Message::Text(t))) = socket.recv().await {
                    if t == "r" {
                        escape = true;
                    }
                }

                eprintln!(
                    "(HMR) {} CONN{} (browser status)   took {:?}",
                    path,
                    conn_id,
                    dur_start.elapsed()
                );

                if escape {
                    eprintln!(
                        "(HMR) {} CONN{} (browser status)   performing full reload",
                        path, conn_id
                    );
                    CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
                    socket.close().await.unwrap();
                    return;
                }

                dur_start = Instant::now();

                if let Some(Ok(ws::Message::Binary(b))) = socket.recv().await {
                    if !TEMPLATES.forms.read().unwrap().contains_key(&ip) {
                        TEMPLATES
                            .forms
                            .write()
                            .unwrap()
                            .insert(ip, Default::default());
                    }

                    let r = TEMPLATES.forms.read().unwrap();
                    let check = r.get(&ip).unwrap();

                    if !check.read().unwrap().contains_key(&endpoint) {
                        check
                            .write()
                            .unwrap()
                            .insert(endpoint.clone(), Default::default());
                    }

                    check.write().unwrap().get_mut(&endpoint).unwrap().indexes = b
                        .chunks(4)
                        .map(<[u8; 4] as TryFrom<&[u8]>>::try_from)
                        .map(Result::unwrap)
                        .map(u32::from_le_bytes)
                        .collect::<VecDeque<_>>();
                }

                eprintln!(
                    "(HMR) {} CONN{} (server form sync) took {:?}",
                    path,
                    conn_id,
                    dur_start.elapsed()
                );
            } else {
                socket
                    .send(ws::Message::Binary(vec![0]))
                    .await
                    .unwrap_or_default();
            }
        }

        eprintln!("(HMR): Closed");
        socket.close().await.unwrap();
    })
}

async fn async_watch<P: AsRef<Path> + std::fmt::Debug>(path: P) -> notify::Result<()> {
    use std::time::Instant;

    use crate::render::reload_template;

    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(Event {
                kind: EventKind::Access(_),
                paths,
                ..
            }) => {
                for path in paths {
                    let dur_start = Instant::now();
                    let path = path.strip_prefix(&*PWD).unwrap();
                    reload_template(&format!(
                        "/{}",
                        path.to_str().unwrap().trim_end_matches(".html.jinja2")
                    ));
                    eprintln!(
                        "\n(HMR) {}       (template render)  took {:?}",
                        path.display(),
                        dur_start.elapsed()
                    );
                    HMR_BROADCAST.0.send(path.display().to_string()).unwrap();
                }
            }
            Err(e) => eprintln!("(HMR): {e:?}"),
            _ => (),
        }
    }

    Ok(())
}

pub(crate) fn watch_templates() {
    tokio::spawn(async {
        if let Err(e) = async_watch("templates").await {
            eprintln!("(HMR): {e}");
        }
    });
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, mpsc::Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = mpsc::channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
