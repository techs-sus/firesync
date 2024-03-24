use super::{HOST, PORT};
use crate::error::Error;
use localtunnel_client::{broadcast, open_tunnel, ClientConfig};
use uuid::Uuid;

#[derive(Debug)]
pub struct Tunnel {
	pub url: Option<String>,
	notify_shutdown: broadcast::Sender<()>,
}

impl Tunnel {
	pub fn new() -> Self {
		Self {
			url: None,
			notify_shutdown: broadcast::channel(1).0,
		}
	}

	pub async fn connect(&mut self) -> Result<(), Error> {
		let subdomain = Uuid::new_v4();
		// todo: serve with axum

		let config = ClientConfig {
			server: Some("https://init.so".to_string()),
			subdomain: Some(subdomain.to_string()),
			local_host: Some(HOST.to_string()),
			local_port: PORT,
			shutdown_signal: self.notify_shutdown.clone(),
			max_conn: 5,
			credential: None,
		};

		let result = open_tunnel(config)
			.await
			.map_err(|_| Error::LocaltunnelConnect)?;

		self.url = Some(result);

		// Shutdown the background tasks by sending a signal.
		// let _ = notify_shutdown.send(());

		Ok(())
	}

	pub fn disconnect(&mut self) -> Result<(), Error> {
		self.url = None;
		let _ = self
			.notify_shutdown
			.send(())
			.map_err(|_| Error::LocaltunnelDisconnect)?;
		Ok(())
	}
}
