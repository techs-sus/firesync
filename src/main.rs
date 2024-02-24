use tui::run;
mod ast_handler;
mod tui;

fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().compact().without_time().init();
	// build(args.input_path, args.output_path)?;
	run().unwrap();
	Ok(())
}
