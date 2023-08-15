pub use futures::channel::mpsc::channel as mpsc_channel;
pub use futures::channel::mpsc::Receiver as MpscReceiver;
pub use futures::channel::mpsc::Sender as MpscSender;
pub use futures::executor::block_on;
pub use futures::{SinkExt, StreamExt};

pub use tokio::spawn;
pub use tokio::sync::broadcast::channel as broadcast_channel;
pub use tokio::sync::broadcast::Receiver as BroadcastReceiver;
pub use tokio::sync::broadcast::Sender as BroadcastSender;

#[cfg(debug_assertions)]
pub fn instant_now() -> tokio::time::Instant {
    tokio::time::Instant::now()
}

pub fn broadcast_subscribe<T>(
    broadcast: &(BroadcastSender<T>, BroadcastReceiver<T>),
) -> BroadcastReceiver<T> {
    broadcast.0.subscribe()
}
