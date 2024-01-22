use std::{fs, io, path::PathBuf};

use clap::Parser;
use darklua_core::{Options, Resources};
use full_moon::{ast, tokenizer, visitors::VisitorMut, ShortString};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Input path of what firesync should bundle
	#[arg(short, long)]
	input_path: PathBuf,

	/// Output path of firesync
	#[arg(short, long)]
	output_path: PathBuf,

	/// Darklua configuration path
	#[arg(short, long, default_value = "./.darklua.json")]
	config_path: PathBuf,
}

#[derive(Default)]
struct PatchVisitor {
	output: PathBuf,
}

fn get_string_from_token_reference(token: &tokenizer::TokenReference) -> Option<&str> {
	match token.token_type() {
		tokenizer::TokenType::Identifier { identifier } => return Some(identifier),
		tokenizer::TokenType::StringLiteral { literal, .. } => return Some(literal),
		_ => {
			return None;
		}
	}
}

/*
	A function call is a call such as "foo(1)". A call is something being called such as "unknown!(1)"
	We want to intercept NS, NLS, so we intercept function calls.
*/

impl VisitorMut for PatchVisitor {
	fn visit_function_call(&mut self, mut node: ast::FunctionCall) -> ast::FunctionCall {
		let prefix = node.prefix();
		let mut suffixes = node.suffixes().collect::<Vec<&ast::Suffix>>();

		// We want foo(1), not foo(1)()
		if suffixes.len() != 1 {
			return node;
		}

		let suffix = suffixes.get_mut(0).unwrap();

		match prefix {
			// This would be ("foo")(1), which is not what we want
			ast::Prefix::Expression(_) => return node,
			ast::Prefix::Name(token_reference) => {
				// We want the token and its type which contains the name of the function
				let function_name = get_string_from_token_reference(token_reference)
					.expect("ast::Prefix::Name should return a valid string TokenReference");
				info!("{function_name}");
				match function_name {
					"NLS" | "NewLocalScript" | "NS" | "NewScript" => {
						// The function signature is (PathBuf, ...) -> LuaSourceContainer
						// We want a function call
						if let ast::Suffix::Call(ast::Call::AnonymousCall(args)) = suffix {
							match args {
								ast::FunctionArgs::Parentheses {
									parentheses,
									arguments,
								} => {
									let file_name = arguments.first();
									if file_name.is_none() {
										return node;
									}
									let file_name = file_name.unwrap().value();
									if let ast::Expression::String(string) = file_name {
										if let Some(file_name) = get_string_from_token_reference(string) {
											let content = fs::read_to_string(self.output.join(file_name))
												.expect("Failed opening file")
												.replace("[==[", "\\[==\\[")
												.replace("]==]", "\\]==\\]");

											let new_token = string.with_token(tokenizer::Token::new(
												tokenizer::TokenType::StringLiteral {
													literal: content.into(),
													multi_line: Some(2),
													quote_type: tokenizer::StringLiteralQuoteType::Brackets,
												},
											));
											let args = arguments.clone();
											let mut new_args = ast::punctuated::Punctuated::new();
											new_args.push(ast::punctuated::Pair::Punctuated(
												ast::Expression::String(new_token),
												tokenizer::TokenReference::new(
													vec![],
													match args.len() > 1 {
														true => tokenizer::Token::new(tokenizer::TokenType::Symbol {
															symbol: tokenizer::Symbol::Comma,
														}),
														false => tokenizer::Token::new(tokenizer::TokenType::Whitespace {
															characters: "".into(),
														}),
													},
													vec![],
												),
											));
											new_args.extend(args.into_pairs().skip(1));
											let new_suffix = ast::Suffix::Call(ast::Call::AnonymousCall(
												ast::FunctionArgs::Parentheses {
													parentheses: parentheses.clone(),
													arguments: new_args,
												},
											));
											suffixes.remove(0);
											suffixes.insert(0, &new_suffix);
											return node
												.clone()
												.with_prefix(ast::Prefix::Name(token_reference.clone()))
												.with_suffixes(
													suffixes
														.into_iter()
														.map(|t| t.to_owned())
														.collect::<Vec<ast::Suffix>>(),
												);
										}
									}
								}
								ast::FunctionArgs::String(string) => {
									//
									let file_name = match get_string_from_token_reference(&string) {
										Some(s) => s,
										None => return node,
									};

									let content = fs::read_to_string(self.output.join(file_name))
										.expect("Failed opening file")
										.replace("[==[", "\\[==\\[")
										.replace("]==]", "\\]==\\]");
									let new_token =
										string.with_token(tokenizer::Token::new(tokenizer::TokenType::StringLiteral {
											literal: content.into(),
											multi_line: Some(2),
											quote_type: tokenizer::StringLiteralQuoteType::Brackets,
										}));
									let new_suffix = ast::Suffix::Call(ast::Call::AnonymousCall(
										ast::FunctionArgs::String(new_token),
									));
									suffixes.remove(0);
									suffixes.insert(0, &new_suffix);
									return node
										.clone()
										.with_prefix(ast::Prefix::Name(token_reference.clone()))
										.with_suffixes(
											suffixes
												.into_iter()
												.map(|t| t.to_owned())
												.collect::<Vec<ast::Suffix>>(),
										);
								}
								ast::FunctionArgs::TableConstructor(_) => return node,
								_ => return node,
							}
						}
					}
					&_ => {}
				}
				return node;
			}
			_ => return node,
		}
	}
}

fn patch_file(file: PathBuf, output: PathBuf) -> anyhow::Result<()> {
	let ast = full_moon::parse(&fs::read_to_string(file.clone())?)?;
	let mut visitor = PatchVisitor::default();
	if output.is_file() {
		visitor.output = output.parent().expect("File has a parent").to_path_buf();
	} else {
		visitor.output = output;
	}
	let ast = visitor.visit_ast(ast);

	fs::write(file, full_moon::print(&ast))?;
	Ok(())
}

fn patch_directory(path: PathBuf) -> anyhow::Result<()> {
	let directory = path.read_dir()?;
	directory.for_each(|entry| match entry {
		Ok(entry) => {
			let file = entry.path();
			match patch_file(file, path.clone()) {
				Ok(..) => {}
				Err(error) => error!("{error}"),
			}
		}
		Err(error) => error!("{error}"),
	});

	Ok(())
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	tracing_subscriber::fmt().compact().without_time().init();

	if args.input_path.is_dir() != args.output_path.is_dir() {
		error!("The input path should be the same type of PathBuf as the output path.");
		return Ok(());
	}

	if !args.input_path.exists() {
		error!("The input path does not exist.");
		return Ok(());
	}

	if !args.output_path.exists() {
		error!("The output path does not exist.");
		return Ok(());
	}

	// TODO: Add field into args for config path
	let result = darklua_core::process(
		&Resources::from_file_system(),
		Options::new(args.input_path.clone())
			.with_configuration_at("./.darklua.json")
			.with_output(args.output_path.clone()),
	)
	.result();

	if result.is_err() {
		let errors = result.unwrap_err();
		error!(
			"{} {} in darklua processing",
			errors.len(),
			match errors.len() > 1 {
				true => "errors",
				false => "error",
			}
		);
		for error in errors {
			error!("{error}");
		}
		return Ok(());
	}

	match args.output_path.clone().is_dir() {
		true => patch_directory(args.output_path.clone())?,
		false => patch_file(args.output_path.clone(), args.output_path)?,
	}

	Ok(())
}
