use std::collections::VecDeque;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

pub use minijinja;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;

use crate::framework::*;
use crate::render::reload_template;
use crate::runtime::*;
use crate::style::STYLE_MAIN_FILE;
use crate::{endpointof, template_dir, template_extension, TEMPLATES};

pub(crate) static PWD: Lazy<PathBuf> =
    Lazy::new(|| std::env::current_dir().unwrap().join(template_dir()));

static HMR_BROADCAST: Lazy<(BroadcastSender<String>, BroadcastReceiver<String>)> =
    Lazy::new(|| broadcast_channel(1));

static CONNECTIONS: AtomicU8 = AtomicU8::new(0);

pub(crate) async fn hmr_handler(mut socket: WebSocket, ip: IpAddr) {
    let conn_id = CONNECTIONS.fetch_add(1, Ordering::SeqCst);

    // It took me. an hour. to find out this single line was breaking HMR.
    // this is its grave.
    //TEMPLATES.forms.lock().insert(ip, Default::default());

    #[allow(unused_mut)]
    let mut rx = broadcast_subscribe(&HMR_BROADCAST);

    while let Ok(path) = rx.recv().await {
        let mut dur_start = instant_now();
        if socket
            .send(websocket_message_text("you up?".into()))
            .await
            .is_err()
        {
            background!(
                "(HMR) {} CONN{} (browser status)   connection closed",
                path,
                conn_id
            );
            CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
            return;
        }

        if path.ends_with(template_extension()) {
            let endpoint = format!("/{}", endpointof(&path).unwrap());

            socket
                .send(websocket_message_text(endpoint.clone()))
                .await
                .unwrap_or_default();

            let mut escape = false;
            if let Some(Ok(msg)) = websocket_recv(&mut socket).await {
                if let Some(t) = websocket_try_as_text(msg) {
                    if t == "r" {
                        escape = true;
                    }
                }
            }

            background!(
                "(HMR) {} CONN{} (browser status)   took {:?}",
                path,
                conn_id,
                dur_start.elapsed()
            );

            if escape {
                background!(
                    "(HMR) {} CONN{} (browser status)   performing full reload",
                    path,
                    conn_id
                );
                CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
                socket.close().await.unwrap();
                return;
            }

            dur_start = instant_now();

            if let Some(Ok(msg)) = websocket_recv(&mut socket).await {
                if let Some(b) = websocket_try_as_binary(msg) {
                    let forms = TEMPLATES.forms.lock();
                    let mut ip_endpoint_history = forms.get(&ip).unwrap().lock();

                    ip_endpoint_history.get_mut(&endpoint).unwrap().indexes = b
                        .chunks(4)
                        .map(<[u8; 4] as TryFrom<&[u8]>>::try_from)
                        .map(Result::unwrap)
                        .map(u32::from_le_bytes)
                        .collect::<VecDeque<_>>();
                }
            }

            background!(
                "(HMR) {} CONN{} (server form sync) took {:?}",
                path,
                conn_id,
                dur_start.elapsed()
            );
        } else {
            socket
                .send(websocket_message_binary(vec![0]))
                .await
                .unwrap_or_default();
        }
    }

    background!("(HMR): Closed");
    socket.close().await.unwrap();
}

async fn async_watch<P: AsRef<Path> + std::fmt::Debug>(watch_path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(watch_path.as_ref(), RecursiveMode::Recursive)?;

    let parent_path = watch_path.as_ref().parent().unwrap();

    while let Some(res) = rx.next().await {
        match res {
            Ok(Event {
                kind: EventKind::Access(_),
                paths,
                ..
            }) => {
                for path in paths {
                    let dur_start = instant_now();
                    let relative_path = path.strip_prefix(parent_path).unwrap();
                    let path = path.strip_prefix(watch_path.as_ref()).unwrap();

                    reload_template(&relative_path.display().to_string());

                    background!(
                        "\n(HMR) {}       (template render)  took {:?}",
                        path.display(),
                        dur_start.elapsed()
                    );

                    websocket_unwrap(HMR_BROADCAST.0.send(path.display().to_string())).await;
                }
            }
            Err(e) => error!("(HMR): {e:?}"),
            _ => (),
        }
    }

    Ok(())
}

pub(crate) fn watch_templates() {
    spawn(async {
        if let Err(e) = async_watch(&*PWD).await {
            error!("(HMR): {e}");
        }
    });
}

pub(crate) fn watch_style() {
    spawn(async {
        let style_file = std::env::current_dir()
            .unwrap()
            .join(STYLE_MAIN_FILE.get().unwrap());

        let style_path = style_file.parent().unwrap();

        if style_path.strip_prefix(&*PWD).is_err() {
            if let Err(e) = async_watch(style_path).await {
                error!("(HMR): {e}");
            }
        }
    });
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, MpscReceiver<notify::Result<Event>>)> {
    #[allow(unused_mut)]
    let (mut tx, rx) = mpsc_channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
