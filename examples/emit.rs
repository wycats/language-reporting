extern crate language_reporting;

extern crate pretty_env_logger;
extern crate structopt;
extern crate termcolor;

use std::io::prelude::*;
use structopt::StructOpt;
use termcolor::{Color, ColorSpec, WriteColor};

use language_reporting::{
    emit, ColorArg, Diagnostic, Label, ReportingFiles, Severity, SimpleReportingFiles, SimpleSpan,
};
use termcolor::StandardStream;

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub struct Opts {
    /// Configure coloring of output
    #[structopt(
        long = "color",
        parse(try_from_str),
        default_value = "auto",
        raw(possible_values = "ColorArg::VARIANTS", case_insensitive = "true")
    )]
    pub color: ColorArg,
}

#[allow(unused)]
fn test(opts: Opts) {
    let mut writer = StandardStream::stderr(opts.color.into());

    // let tree = tree! {
    //     <Message args={foo}>
    // };

    writer
        .set_color(
            ColorSpec::new()
                .set_bold(true)
                .set_intense(true)
                .set_fg(Some(Color::White)),
        )
        .unwrap();
    writeln!(writer, "hello world").unwrap();
    writer.reset().unwrap();
}

fn main() {
    pretty_env_logger::init();
    let opts = Opts::from_args();

    let mut files = SimpleReportingFiles::default();

    let source = r##"
(define test 123)
(+ test "")
()
"##;
    let file = files.add("test", source);

    let str_start = files.byte_index(file, 2, 8).unwrap();
    let error = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
        .with_label(
            Label::new_primary(SimpleSpan::new(file, str_start, str_start + 2))
                .with_message("Expected integer but got string"),
        )
        .with_label(
            Label::new_secondary(SimpleSpan::new(file, str_start, str_start + 2))
                .with_message("Expected integer but got string"),
        )
        .with_code("E0001");

    let line_start = files.byte_index(file, 2, 0).unwrap();
    let warning = Diagnostic::new(
        Severity::Warning,
        "`+` function has no effect unless its result is used",
    )
    .with_label(Label::new_primary(SimpleSpan::new(
        file,
        line_start,
        line_start + 11,
    )));

    let no_file = Diagnostic::new(Severity::Help, "Great job!");

    let diagnostics = [error, warning, no_file];

    let writer = StandardStream::stderr(opts.color.into());
    for diagnostic in &diagnostics {
        emit(
            &mut writer.lock(),
            &files,
            &diagnostic,
            &language_reporting::DefaultConfig,
        )
        .unwrap();
        println!();
    }

    test(Opts::from_args());
}
