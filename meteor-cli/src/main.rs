use std::path::PathBuf;

use ariadne::{Label, Report, Source};
use clap::{self, Parser};
use meteor::{parser::src::SourceId, runtime::Chip};

#[derive(Parser)]
#[command(version, about, author)]
struct Args {
    #[arg(long, short)]
    file: PathBuf,

    /// Prints out the AST contents instead of evaluating
    #[arg(long, short)]
    dump_ast: bool,
}

fn main() {
    let args = Args::parse();

    let Ok(contents) = std::fs::read_to_string(&args.file) else {
        eprintln!("Failed to open file {}", args.file.to_string_lossy());
        return;
    };

    let prog_ctx = SourceId::new(args.file.to_string_lossy());

    match meteor::parse(prog_ctx.clone(), &contents) {
        Ok(ast) => {
            if args.dump_ast {
                println!("{}", serde_json::to_string_pretty(&ast).unwrap());
                return;
            }

            let mut chip = Chip::new(ast);

            match chip.run() {
                Ok(value) => {
                    println!("Exited with value \"{value}\"");
                }
                Err(err) => {
                    Report::build(ariadne::ReportKind::Error, err.span().clone())
                        .with_message(err.reason())
                        .with_label(Label::new(err.span().clone()).with_message(format!("{err}")))
                        .finish()
                        .eprint((prog_ctx, Source::from(&contents)))
                        .unwrap();
                }
            }
        }

        Err(error) => {
            let prog_name = args.file.to_string_lossy();

            for err in error {
                Report::build(ariadne::ReportKind::Error, (&prog_name, err.span().range()))
                    .with_message(err.reason())
                    .with_label(
                        Label::new((&prog_name, err.span().range())).with_message(err.reason()),
                    )
                    .finish()
                    .print((&prog_name, Source::from(&contents)))
                    .unwrap();
            }
        }
    }
}
