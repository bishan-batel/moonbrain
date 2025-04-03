use std::path::PathBuf;

use ariadne::{Label, Report, Source};
use clap::{self, Parser};
use meteor::{parser::src::SourceId, runtime::Chip, semantic};

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

            let prog_name = args.file.to_string_lossy();

            for diagnoses in semantic::analyze(&ast).diagnostics {
                let kind = match diagnoses.severity {
                    semantic::Severity::Hint => ariadne::ReportKind::Advice,
                    semantic::Severity::Warning => ariadne::ReportKind::Warning,
                    semantic::Severity::Error => ariadne::ReportKind::Error,
                };
                Report::build(kind, (&prog_name, diagnoses.span.range()))
                    .with_message(diagnoses.reason())
                    .with_label(
                        Label::new((&prog_name, diagnoses.span.range()))
                            .with_message(diagnoses.reason()),
                    )
                    .finish()
                    .print((&prog_name, Source::from(&contents)))
                    .unwrap();
            }

            let mut chip = Chip::new(ast);

            match chip.run() {
                Ok(value) => {
                    println!("Exited with value \"{value}\"");
                }
                Err(err) => {
                    err.write((prog_ctx, Source::from(&contents)), std::io::stderr());
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
