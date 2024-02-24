// https://github.com/rhysd/tui-textarea/blob/main/examples/single_line.rs
use crate::ast_handler;
use crossterm::{
	event::{self, Event, KeyCode, KeyEventKind, KeyEventState},
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{error::Error, io, ops::Add};
use tui_textarea::TextArea;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

enum SelectedTextArea {
	Input,
	Output,
	Configuration,
}

impl ToString for SelectedTextArea {
	fn to_string(&self) -> String {
		match self {
			SelectedTextArea::Input => "Input path",
			SelectedTextArea::Output => "Output path",
			SelectedTextArea::Configuration => "Configuration path",
		}
		.to_string()
	}
}

impl SelectedTextArea {
	fn value(&self) -> u8 {
		match self {
			SelectedTextArea::Input => 0,
			SelectedTextArea::Output => 1,
			SelectedTextArea::Configuration => 2,
		}
	}

	fn from_value(value: u8) -> Self {
		match value {
			0 => SelectedTextArea::Input,
			1 => SelectedTextArea::Output,
			2 => SelectedTextArea::Configuration,
			_ => panic!("invalid value"),
		}
	}
}

enum KeyboardMode {
	Normal,
	Editing,
}

struct App<'a> {
	keyboard_mode: KeyboardMode,
	selected_textarea: SelectedTextArea,

	input_textarea: TextArea<'a>,
	output_textarea: TextArea<'a>,
	configuration_textarea: TextArea<'a>,
}

pub fn run() -> Result<()> {
	let mut terminal = init_terminal()?;

	let mut app = App {
		keyboard_mode: KeyboardMode::Normal,
		selected_textarea: SelectedTextArea::Input,
		input_textarea: TextArea::new(vec![]),
		output_textarea: TextArea::new(vec![]),
		configuration_textarea: TextArea::new(vec![]),
	};
	let res = run_tui(&mut terminal, &mut app);

	reset_terminal()?;

	if let Err(err) = res {
		println!("{err:?}");
	}

	Ok(())
}

/// Initializes the terminal.
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
	crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
	enable_raw_mode()?;

	let backend = CrosstermBackend::new(io::stdout());

	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;

	Ok(terminal)
}

/// Resets the terminal.
fn reset_terminal() -> Result<()> {
	disable_raw_mode()?;
	crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

	Ok(())
}

/// Runs the TUI loop.
fn run_tui<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
	loop {
		terminal.draw(|f| ui(f, app))?;

		if let Event::Key(key) = event::read()? {
			if key.kind == KeyEventKind::Press {
				match app.keyboard_mode {
					KeyboardMode::Editing => match key.code {
						KeyCode::Down => {
							app.selected_textarea =
								SelectedTextArea::from_value(app.selected_textarea.value().add(1).clamp(0, 2));
						}
						KeyCode::Up => {
							app.selected_textarea =
								SelectedTextArea::from_value(app.selected_textarea.value().saturating_sub(1));
						}
						KeyCode::Enter => {
							app.keyboard_mode = KeyboardMode::Normal;
						}
						KeyCode::Esc => {
							app.keyboard_mode = KeyboardMode::Normal;
						}
						_ => {
							match app.selected_textarea {
								SelectedTextArea::Input => app.input_textarea.input(Event::Key(key)),
								SelectedTextArea::Output => app.output_textarea.input(Event::Key(key)),
								SelectedTextArea::Configuration => {
									app.configuration_textarea.input(Event::Key(key))
								}
							};
						}
					},
					KeyboardMode::Normal => {
						match key.code {
							// quit the loop
							KeyCode::Char('q') => return Ok(()),
							KeyCode::Char('e') => {
								app.keyboard_mode = KeyboardMode::Editing;
							}
							KeyCode::Char('r') => {
								// rebuild
								// input textbox
								// output textbox
								// ast_handler::build(input, output)
							}
							_ => {}
						}
					}
				}
			}
		}
	}
}

fn deselect(textarea: &mut TextArea<'_>, selected: SelectedTextArea) {
	textarea.set_cursor_line_style(Style::default());
	textarea.set_cursor_style(Style::default());
	textarea.set_block(
		Block::default()
			.borders(Borders::ALL)
			.style(Style::default().fg(Color::DarkGray))
			.title(format!(
				"{} (not selected, use arrows)",
				selected.to_string()
			)),
	);
}

fn select(textarea: &mut TextArea<'_>, selected: SelectedTextArea) {
	textarea.set_cursor_line_style(Style::default());
	textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
	textarea.set_block(
		Block::default()
			.borders(Borders::ALL)
			.style(Style::default())
			.title(format!("{} (selected)", selected.to_string())),
	);
}

/// Render the TUI.
fn ui(f: &mut Frame, app: &mut App) {
	let text = vec![
		Line::from("q to quit"),
		Line::from("r to rebuild"),
		match app.keyboard_mode {
			KeyboardMode::Editing => Line::from("enter/esc to exit editing"),
			KeyboardMode::Normal => Line::from("e to edit"),
		},
	];

	let b = Block::default()
		.title("keybinds")
		.borders(Borders::ALL)
		.border_style(Style::new().blue());

	let p = Paragraph::new(text)
		.block(b.clone())
		.alignment(Alignment::Left);
	let layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
		.split(f.size());
	let paragraph_chunk = layout[0];
	let config_chunk = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Ratio(1, 3); 3])
		.split(layout[1]);
	let input_area = config_chunk[0];
	let output_area = config_chunk[1];
	let configuration_area = config_chunk[2];
	let mut input_textarea = &mut app.input_textarea;
	let mut output_textarea = &mut app.output_textarea;
	let mut configuration_textarea = &mut app.configuration_textarea;

	match app.selected_textarea {
		SelectedTextArea::Input => {
			select(&mut input_textarea, SelectedTextArea::Input);
			deselect(&mut output_textarea, SelectedTextArea::Output);
			deselect(&mut configuration_textarea, SelectedTextArea::Configuration);
		}
		SelectedTextArea::Output => {
			select(&mut output_textarea, SelectedTextArea::Output);
			deselect(&mut input_textarea, SelectedTextArea::Input);
			deselect(&mut configuration_textarea, SelectedTextArea::Configuration);
		}
		SelectedTextArea::Configuration => {
			select(&mut configuration_textarea, SelectedTextArea::Configuration);
			deselect(&mut input_textarea, SelectedTextArea::Input);
			deselect(&mut output_textarea, SelectedTextArea::Output);
		}
	}

	f.render_widget(p, paragraph_chunk);
	f.render_widget(input_textarea.widget(), input_area);
	f.render_widget(output_textarea.widget(), output_area);
	f.render_widget(configuration_textarea.widget(), configuration_area);
}
