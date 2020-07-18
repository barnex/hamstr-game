use crate::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Constructs a multi-producer, multi-consumer channel.
/// Simlar to stdd:sync::mpsc, but allowing for multiple consuming threads.
pub fn mpmc_channel<T>() -> (Sender<T>, MReceiver<T>) {
	let (sender, receiver) = channel();
	(sender, MReceiver(Arc::new(Mutex::new(receiver))))
}

/// Muti-consumer receiver.
/// Like mpsc::Receiver, but allowing for multiple consuming threads.
#[derive(Clone)]
pub struct MReceiver<T>(Arc<Mutex<Receiver<T>>>);

impl<T> MReceiver<T> {
	/// Like mpsc::Receiver::recv, but allowing for multiple consuming threads.
	pub fn recv(&mut self) -> Result<T> {
		Ok(self.0.lock().unwrap().recv()?)
	}
}
/// Like mpsc::Receiver::Iterator, but allowing for multiple consuming threads.
impl<T> Iterator for MReceiver<T> {
	type Item = T;
	fn next(&mut self) -> Option<T> {
		self.0.lock().unwrap().recv().ok()
	}
}
