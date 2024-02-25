use iced::widget::{self, button, column, container, pick_list, row, slider, text, text_input};
use iced::{executor, Font, Theme};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};

pub fn run() -> iced::Result {
	App::run(Settings::default())
}

enum BuildStatus {
	Unbuilt,
	Building,
	Built,
	Rebuilding,
}

struct App {
	build_status: BuildStatus,
}

#[derive(Debug, Clone)]
pub enum Error {}

#[derive(Debug, Clone)]
enum Message {
	Build,
	FinishedBuilding(),
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
				self.build_status = BuildStatus::Building;

				// Command::perform(async {}, Message::FinishedBuilding)
				Command::none()
			}
			Message::FinishedBuilding() => Command::none(),
		}
	}

	fn view<'a>(&'a self) -> Element<'a, Message> {
		let content = column([button("Build").on_press(Message::Build).into()]);
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
