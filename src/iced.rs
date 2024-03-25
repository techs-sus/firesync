// TODO: Implement config loading (see firesync.json)

use crate::ast_handler::build;
use crate::error;
use crate::server::Server;
use iced::widget::{
	button, column, container, pick_list, row, slider, space, text, text_input, Row,
};
use iced::{executor, subscription, Font, Renderer, Theme};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};
use native_dialog::FileDialog;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn run() -> iced::Result {
	App::run(Settings::default())
}

#[derive(Debug)]
enum BuildStatus {
	Unbuilt,
	Building,
	ErrorBuilding(Arc<Vec<error::Error>>),
	Built,
	Rebuilding,
}

#[derive(Debug)]
enum ServerStatus {
	Running,
	Loading,
	Stopped,
}

#[derive(Clone, Default, Deserialize)]
struct Config {
	input: PathBuf,
	output: PathBuf,
	darklua_configuration: PathBuf,
}

struct App {
	build_status: BuildStatus,
	server_status: ServerStatus,
	config: Config,
	config_path: PathBuf,
	development_server: Arc<Mutex<Server>>,
}

#[derive(Debug, Clone)]
enum Message {
	Build,
	FinishedBuilding(Result<(), Arc<Vec<error::Error>>>),
	StartDevelopmentServer,
	StartedDevelopmentServer,
	StopDevelopmentServer,
	StoppedDevelopmentServer,
	PickConfigurationPath,
	FinishedPicking(Option<Option<PathBuf>>),
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
				server_status: ServerStatus::Stopped,
				config: Config::default(),
				config_path: PathBuf::new(),
				development_server: Arc::new(Mutex::new(Server::new())),
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
				let (input, output) = (self.config.input.clone(), self.config.output.clone());
				Command::perform(async { build(input, output) }, |result| {
					Message::FinishedBuilding(result.map_err(Arc::new))
				})
			}
			Message::FinishedBuilding(option) => {
				match option {
					Ok(..) => self.build_status = BuildStatus::Built,
					Err(errors) => self.build_status = BuildStatus::ErrorBuilding(errors),
				};
				Command::none()
			}
			Message::PickConfigurationPath => Command::perform(
				async { FileDialog::new().show_open_single_file().ok() },
				Message::FinishedPicking,
			),
			Message::FinishedPicking(path) => {
				if let Some(Some(path)) = path {
					self.config_path = path.clone();
					// TODO: We shouldn't panic if its invalid json
					self.config =
						serde_json::from_str(&std::fs::read_to_string(path).expect("To read firesync.json"))
							.expect("Valid firesync.json");
				}
				Command::none()
			}
			Message::StartDevelopmentServer => {
				self.server_status = ServerStatus::Loading;

				let server = self.development_server.clone();
				Command::perform(async move { server.lock().await.start().await }, |_| {
					Message::StartedDevelopmentServer
				})
			}
			Message::StartedDevelopmentServer => {
				self.server_status = ServerStatus::Running;
				Command::none()
			}
			Message::StopDevelopmentServer => {
				self.server_status = ServerStatus::Loading;

				let server = self.development_server.clone();
				Command::perform(async move { server.lock().await.stop() }, |_| {
					Message::StoppedDevelopmentServer
				})
			}
			Message::StoppedDevelopmentServer => {
				self.server_status = ServerStatus::Stopped;
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<'_, Message> {
		let content = column([
			// TODO: Add more configuration options
			file_picker(self.config_path.clone()).into(),
			row([
				button(text("Build!").font(Font::MONOSPACE))
					.on_press(Message::Build)
					.into(),
				// TODO: Add task log + build log from development server (subscriptions)
				// TODO: Also add notify-rs log into UI
				match self.server_status {
					ServerStatus::Running => button(text("Stop server").font(Font::MONOSPACE))
						.on_press(Message::StopDevelopmentServer)
						.into(),
					ServerStatus::Loading => button(text("Server loading").font(Font::MONOSPACE)).into(),
					ServerStatus::Stopped => button(text("Start server").font(Font::MONOSPACE))
						.on_press(Message::StartDevelopmentServer)
						.into(),
				},
			])
			.spacing(5)
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

	// TODO: We want a channel subscription (https://github.com/iced-rs/iced/blob/0.12/examples/websocket/src/echo.rs#L16)
	// fn subscription(&self) -> Subscription<Self::Message> {
	// 	subscription::unfold(0, "", move |state| async { (Message::Build, "") })
	// }

	fn theme(&self) -> Self::Theme {
		Theme::GruvboxDark
	}
}

fn file_picker<'a>(path: PathBuf) -> Row<'a, Message, Theme, Renderer> {
	row([
		button(text("Select a firesync.json").font(Font::MONOSPACE))
			.on_press(Message::PickConfigurationPath)
			.into(),
		text(path.display()).font(Font::MONOSPACE).into(),
	])
	.spacing(10)
	.align_items(Alignment::Center)
}
