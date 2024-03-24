mod axum;
mod localtunnel;

use self::{axum::AxumServer, localtunnel::Tunnel};
use crate::{ast_handler::build, error::Error};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, DebouncedEvent};
use std::{
	path::{Path, PathBuf},
	sync::Arc,
	time::Duration,
};
use tokio::{
	sync::{mpsc, Mutex},
	task::AbortHandle,
};

pub const PORT: u16 = 3000;
pub const HOST: &str = "localhost";

#[derive(Debug, Clone)]
pub enum Task {
	Build,
}

pub type Debouncer = notify_debouncer_full::Debouncer<
	notify::ReadDirectoryChangesWatcher,
	notify_debouncer_full::FileIdMap,
>;

#[derive(Debug)]
pub struct Server {
	pub tunnel: Tunnel,
	pub axum_server: AxumServer,
	pub task_sender: Arc<mpsc::Sender<Task>>,
	task_receiver: Arc<Mutex<mpsc::Receiver<Task>>>,
	task_thread_handle: Option<AbortHandle>,

	notify_debouncer: Option<Debouncer>,
	input_directory: PathBuf,
	output_directory: PathBuf,
}

impl Server {
	pub fn new() -> Self {
		let (sender, receiver) = mpsc::channel(32);
		Server {
			tunnel: Tunnel::new(),
			axum_server: AxumServer::new(),
			task_sender: Arc::new(sender),
			task_receiver: Arc::new(Mutex::new(receiver)),
			task_thread_handle: None,

			notify_debouncer: None,
			input_directory: PathBuf::new(),
			output_directory: PathBuf::new(),
		}
	}

	pub fn start_task_thread(&self) -> AbortHandle {
		// TODO: Allow for dynamic path changing in UI
		let input_directory = self.input_directory.clone();
		let output_directory = self.output_directory.clone();
		let receiver = self.task_receiver.clone();
		tokio::spawn(async move {
			while let Some(task) = receiver.lock().await.recv().await {
				match task {
					Task::Build => {
						if let Err(e) = build(input_directory.clone(), output_directory.clone()) {
							tracing::error!("error(s?) while building in thread: {e:?}")
						};
					}
				}
			}
		})
		.abort_handle()
	}

	pub async fn start(&mut self) -> Result<(), Error> {
		self.axum_server.start().await;
		self.tunnel.connect().await?;

		self.task_thread_handle = Some(self.start_task_thread());
		let sender = self.task_sender.clone();
		let mut debouncer: Debouncer = new_debouncer(
			Duration::from_secs(2),
			None,
			move |result: DebounceEventResult| match result {
				Ok(events) => events.iter().for_each(|event| match &event.event.kind {
					EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
						// send build task
						let _ = sender.blocking_send(Task::Build);
					}
					_ => {}
				}),
				Err(errors) => errors
					.iter()
					.for_each(|error| tracing::error!("notify-rs errors: {error:?}")),
			},
		)?;

		debouncer
			.watcher()
			.watch(&self.input_directory, RecursiveMode::Recursive)?;
		self.notify_debouncer = Some(debouncer);

		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), Error> {
		self.tunnel.disconnect()?;
		self.axum_server.stop();
		// Drop will stop the notify debouncer
		self.notify_debouncer = None;

		Ok(())
	}
}
