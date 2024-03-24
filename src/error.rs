use darklua_core::DarkluaError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Error in darklua: {0:?}")]
	Darklua(Vec<DarkluaError>),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Error in full_moon (AST): {0}")]
	FullMoon(#[from] full_moon::Error),

	#[error("Error in localtunnel")]
	Localtunnel,

	#[error("Error while connecting to localtunnel")]
	LocaltunnelConnect,

	#[error("Error while disconnecting to localtunnel")]
	LocaltunnelDisconnect,

	#[error("Error while adding paths to fs watcher: {0}")]
	NotifyAddPaths(#[from] notify::Error),
}

impl From<Vec<DarkluaError>> for Error {
	fn from(value: Vec<DarkluaError>) -> Self {
		Self::Darklua(value)
	}
}
