use darklua_core::DarkluaError;

#[derive(Debug, Clone)]
pub enum Error {
	Darklua(Vec<DarkluaError>),
	Io(std::io::ErrorKind),
	FullMoon(full_moon::Error),
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::Io(value.kind())
	}
}

impl From<Vec<DarkluaError>> for Error {
	fn from(value: Vec<DarkluaError>) -> Self {
		Self::Darklua(value)
	}
}

impl From<full_moon::Error> for Error {
	fn from(value: full_moon::Error) -> Self {
		Self::FullMoon(value)
	}
}
