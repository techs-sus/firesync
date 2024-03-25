use super::PORT;
use axum::{routing::get, Router};
use tokio::task::AbortHandle;
use tracing::info;

#[derive(Debug)]
pub struct AxumServer {
	abort_handle: Option<AbortHandle>,
}

impl AxumServer {
	pub fn new() -> Self {
		Self { abort_handle: None }
	}

	pub async fn start(&mut self) {
		let app = Router::new().route("/", get(|| async { "Hello, World!" }));

		let handle = tokio::spawn(async {
			let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}"))
				.await
				.unwrap();
			info!("starting axum thread");
			axum::serve(listener, app).await.unwrap();
		});
		self.abort_handle = Some(handle.abort_handle());
	}

	pub fn stop(&mut self) {
		if let Some(handle) = &self.abort_handle {
			info!("aborting axum thread");
			handle.abort();
			self.abort_handle = None;
		}
	}
}
