use std::path::PathBuf;

use ariadne::{Label, Report, Source};
use clap::{self, Parser};
use eyre::Error;
use meteor::runtime::Chip;

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

    match meteor::parse(&contents) {
        Ok(ast) => {
            if args.dump_ast {
                println!("{}", serde_json::to_string_pretty(&ast).unwrap());
                return;
            }

            let mut chip = Chip::new(ast);

            let x = chip.run().unwrap();

            println!("Exited with value {x:#?}");
        }

        Err(error) => {
            let prog_name = args.file.to_string_lossy();

            for err in error {
                Report::build(
                    ariadne::ReportKind::Error,
                    (&prog_name, err.span().into_range()),
                )
                .with_message(err.reason())
                .with_label(
                    Label::new((&prog_name, err.span().into_range())).with_message(err.reason()),
                )
                .finish()
                .print((&prog_name, Source::from(&contents)))
                .unwrap();
            }
        }
    }
}
