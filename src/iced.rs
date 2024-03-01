use std::path::{Path, PathBuf};

use crate::ast_handler::build;
use crate::error;
use iced::widget::{
	self, button, column, container, pick_list, row, slider, space, text, text_input, Row,
};
use iced::{executor, Font, Renderer, Theme};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};
use native_dialog::FileDialog;
use tracing::info;

pub fn run() -> iced::Result {
	App::run(Settings::default())
}

#[derive(Debug)]
enum BuildStatus {
	Unbuilt,
	Building,
	ErrorBuilding(Vec<error::Error>),
	Built,
	Rebuilding,
}

struct App {
	build_status: BuildStatus,

	input_path: PathBuf,
	output_path: PathBuf,
	config_path: PathBuf,
}

#[derive(Debug, Clone)]
enum PickedDirectory {
	Input,
	Output,
	Config,
}

#[derive(Debug, Clone)]
enum Message {
	Build,
	FinishedBuilding(Result<(), Vec<error::Error>>),
	PickDirectory(PickedDirectory),
	FinishedPicking((PickedDirectory, Option<Option<PathBuf>>)),
}

impl Application for App {
	type Executor = executor::Default;
	type Message = Message;
	type Theme = iced::Theme;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(
			App {
				build_status: BuildStatus::Unbuilt,
				input_path: PathBuf::new(),
				output_path: PathBuf::new(),
				config_path: PathBuf::new(),
			},
			Command::none(),
		)
	}

	fn title(&self) -> String {
		String::from("Firesync")
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::Build => {
				if let BuildStatus::Built = self.build_status {
					self.build_status = BuildStatus::Rebuilding;
				} else {
					self.build_status = BuildStatus::Building;
				}
				let (input, output) = (self.input_path.clone(), self.output_path.clone());
				Command::perform(async { build(input, output) }, Message::FinishedBuilding)
				// Command::none()
			}
			Message::FinishedBuilding(option) => {
				match option {
					Ok(..) => self.build_status = BuildStatus::Built,
					Err(errors) => self.build_status = BuildStatus::ErrorBuilding(errors),
				};
				Command::none()
			}
			Message::PickDirectory(directory) => {
				if let PickedDirectory::Config = directory {
					Command::perform(
						async { (directory, FileDialog::new().show_open_single_file().ok()) },
						Message::FinishedPicking,
					)
				} else {
					Command::perform(
						async { (directory, FileDialog::new().show_open_single_dir().ok()) },
						Message::FinishedPicking,
					)
				}
			}
			Message::FinishedPicking((directory, path)) => {
				if let Some(path) = path {
					if let Some(path) = path {
						match directory {
							PickedDirectory::Input => self.input_path = path,
							PickedDirectory::Output => self.output_path = path,
							PickedDirectory::Config => self.config_path = path,
						}
					}
				}
				Command::none()
			}
		}
	}

	fn view<'a>(&'a self) -> Element<'a, Message> {
		let content = column([
			// todo add more
			file_picker(self.input_path.clone(), PickedDirectory::Input).into(),
			file_picker(self.output_path.clone(), PickedDirectory::Output).into(),
			file_picker(self.config_path.clone(), PickedDirectory::Config).into(),
			row([button(text("Build!").font(Font::MONOSPACE))
				.on_press(Message::Build)
				.into()])
			.into(),
			text(format!("Status: {:?}", self.build_status))
				.font(Font::MONOSPACE)
				.into(),
		])
		.spacing(5);
		let container = container(content)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.center_y();

		container.into()
	}

	fn theme(&self) -> Self::Theme {
		Theme::GruvboxDark
	}
}

fn file_picker<'a>(path: PathBuf, directory: PickedDirectory) -> Row<'a, Message, Theme, Renderer> {
	let text_content = format!(
		"Pick the {}",
		match directory.clone() {
			PickedDirectory::Input => "input directory",
			PickedDirectory::Output => "output directory",
			PickedDirectory::Config => "configuration file",
		}
	);
	row([
		button(text(text_content).font(Font::MONOSPACE))
			.on_press(Message::PickDirectory(directory.clone()))
			.into(),
		text(path.display()).font(Font::MONOSPACE).into(),
	])
	.spacing(10)
	.align_items(Alignment::Center)
}
