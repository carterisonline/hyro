pub use async_channel::bounded as mpsc_channel;
pub use async_channel::bounded as broadcast_channel;
pub use async_channel::Receiver as MpscReceiver;
pub use async_channel::Receiver as BroadcastReceiver;
pub use async_channel::Sender as MpscSender;
pub use async_channel::Sender as BroadcastSender;
pub use async_io::block_on;
pub use futures_lite::StreamExt;

#[cfg(debug_assertions)]
pub fn spawn<F: futures_lite::Future + Send + 'static>(future: F)
where
    F::Output: Send + 'static,
{
    async_global_executor::spawn(future).detach();
}

#[cfg(debug_assertions)]
pub fn broadcast_subscribe<T>(
    broadcast: &(BroadcastSender<T>, BroadcastReceiver<T>),
) -> &BroadcastReceiver<T> {
    &broadcast.1
}

#[cfg(debug_assertions)]
pub fn instant_now() -> std::time::Instant {
    std::time::Instant::now()
}
