#![feature(rust_2018_preview)]
extern crate codespan;
extern crate language_reporting;

#[macro_use]
extern crate structopt;
extern crate pretty_env_logger;
extern crate termcolor;

use std::io::prelude::*;
use structopt::StructOpt;
use termcolor::{Color, ColorSpec, WriteColor};

use codespan::{CodeMap, Span};
use language_reporting::{emit, ColorArg, Diagnostic, Label, Severity};
use termcolor::StandardStream;

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub struct Opts {
    /// Configure coloring of output
    #[structopt(
        long = "color",
        parse(try_from_str),
        default_value = "auto",
        raw(
            possible_values = "ColorArg::VARIANTS",
            case_insensitive = "true"
        )
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
        ).unwrap();
    writeln!(writer, "hello world").unwrap();
    writer.reset().unwrap();
}

fn main() {
    pretty_env_logger::init();
    let opts = Opts::from_args();

    let mut code_map = CodeMap::new();

    let source = r##"
(define test 123)
(+ test "")
()
"##;
    let file_map = code_map.add_filemap("test".into(), source.to_string());

    let str_start = file_map.byte_index(2.into(), 8.into()).unwrap();
    let error = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
        .with_label(
            Label::new_primary(Span::from_offset(str_start, 2.into()))
                .with_message("Expected integer but got string"),
        ).with_label(
            Label::new_secondary(Span::from_offset(str_start, 2.into()))
                .with_message("Expected integer but got string"),
        ).with_code("E0001");

    let line_start = file_map.byte_index(2.into(), 0.into()).unwrap();
    let warning = Diagnostic::new(
        Severity::Warning,
        "`+` function has no effect unless its result is used",
    ).with_label(Label::new_primary(Span::from_offset(line_start, 11.into())));

    let no_file = Diagnostic::new(Severity::Help, "Great job!");

    let bogus_span = Diagnostic::new(Severity::Bug, "Something really bad went wrong").with_label(
        Label::new_primary(Span::from_offset(150.into(), 250.into())).with_message("YIKES"),
    );

    let diagnostics = [error, warning, no_file, bogus_span];

    let writer = StandardStream::stderr(opts.color.into());
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &code_map, &diagnostic).unwrap();
        println!();
    }

    test(Opts::from_args());
}
