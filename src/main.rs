mod ast_handler;
mod error;
mod iced;
mod server;

fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().compact().without_time().init();
	// build(args.input_path, args.output_path)?;
	iced::run().unwrap();
	Ok(())
}
