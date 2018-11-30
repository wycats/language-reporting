use crate::components;
use crate::diagnostic::Diagnostic;
use crate::span::ReportingFiles;

use log;
use render_tree::{Component, Render, Stylesheet};
use std::path::Path;
use std::{fmt, io};
use termcolor::WriteColor;

pub fn emit<'doc, W, Files: ReportingFiles>(
    writer: W,
    files: &'doc Files,
    diagnostic: &'doc Diagnostic<Files::Span>,
    config: &'doc dyn Config,
) -> io::Result<()>
where
    W: WriteColor,
{
    DiagnosticWriter { writer }.emit(DiagnosticData {
        files,
        diagnostic,
        config,
    })
}

struct DiagnosticWriter<W> {
    writer: W,
}

impl<W> DiagnosticWriter<W>
where
    W: WriteColor,
{
    fn emit(mut self, data: DiagnosticData<'doc, impl ReportingFiles>) -> io::Result<()> {
        let document = Component(components::Diagnostic, data).into_fragment();

        let styles = Stylesheet::new()
            .add("** header **", "weight: bold")
            .add("bug ** primary", "fg: red")
            .add("error ** primary", "fg: red")
            .add("warning ** primary", "fg: yellow")
            .add("note ** primary", "fg: green")
            .add("help ** primary", "fg: cyan")
            .add("** secondary", "fg: blue")
            .add("** gutter", "fg: blue");

        if log::log_enabled!(log::Level::Debug) {
            document.debug_write(&mut self.writer, &styles)?;
        }

        document.write_with(&mut self.writer, &styles)?;

        Ok(())
    }
}

pub trait Config: std::fmt::Debug {
    fn filename(&self, path: &Path) -> String;
}

#[derive(Debug)]
pub struct DefaultConfig;

impl Config for DefaultConfig {
    fn filename(&self, path: &Path) -> String {
        format!("{}", path.display())
    }
}

#[derive(Debug)]
crate struct DiagnosticData<'doc, Files: ReportingFiles> {
    crate files: &'doc Files,
    crate diagnostic: &'doc Diagnostic<Files::Span>,
    crate config: &'doc dyn Config,
}

pub fn format(f: impl Fn(&mut fmt::Formatter) -> fmt::Result) -> impl fmt::Display {
    struct Display<F>(F);

    impl<F> fmt::Display for Display<F>
    where
        F: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            (self.0)(f)
        }
    }
    Display(f)
}

#[cfg(test)]
mod default_emit_smoke_tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, Label};
    use crate::simple::*;
    use crate::termcolor::Buffer;
    use crate::Severity;

    use regex;
    use render_tree::stylesheet::ColorAccumulator;
    use unindent::unindent;

    fn emit_with_writer<W: WriteColor>(mut writer: W) -> W {
        let mut files = SimpleReportingFiles::default();

        let source = unindent(
            r##"
                (define test 123)
                (+ test "")
                ()
            "##,
        );

        let file = files.add("test", source);

        let str_start = files.byte_index(file, 1, 8).unwrap();
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

        let line_start = files.byte_index(file, 1, 0).unwrap();
        let warning = Diagnostic::new(
            Severity::Warning,
            "`+` function has no effect unless its result is used",
        )
        .with_label(Label::new_primary(SimpleSpan::new(
            file,
            line_start,
            line_start + 11,
        )));

        let diagnostics = [error, warning];

        for diagnostic in &diagnostics {
            emit(&mut writer, &files, &diagnostic, &super::DefaultConfig).unwrap();
        }

        writer
    }

    #[test]
    fn test_no_color() {
        assert_eq!(
            String::from_utf8_lossy(&emit_with_writer(Buffer::no_color()).into_inner()),
            unindent(&format!(
                r##"
                    error[E0001]: Unexpected type in `+` application
                    - test:2:9
                    2 | (+ test "")
                      |         ^^ Expected integer but got string
                    - test:2:9
                    2 | (+ test "")
                      |         -- Expected integer but got string
                    warning: `+` function has no effect unless its result is used
                    - test:2:1
                    2 | (+ test "")
                      | ^^^^^^^^^^^
                "##,
            )),
        );
    }

    #[test]
    fn test_color() {
        assert_eq!(
            emit_with_writer(ColorAccumulator::new()).to_string(),

            normalize(
                r#"
                   {fg:Red bold bright} $$error[E0001]{bold bright}: Unexpected type in `+` application{/}
                                        $$- test:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Red}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Red}^^ Expected integer but got string{/}
                                        $$- test:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Cyan}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Cyan}-- Expected integer but got string{/}
                {fg:Yellow bold bright} $$warning{bold bright}: `+` function has no effect unless its result is used{/}
                                        $$- test:2:1
                              {fg:Cyan} $$2 | {fg:Yellow}(+ test ""){/}
                              {fg:Cyan} $$  | {fg:Yellow}^^^^^^^^^^^{/}
            "#
            )
        );
    }

    fn split_line<'a>(line: &'a str, by: &str) -> (&'a str, &'a str) {
        let mut splitter = line.splitn(2, by);
        let first = splitter.next().unwrap_or("");
        let second = splitter.next().unwrap_or("");
        (first, second)
    }

    fn normalize(s: impl AsRef<str>) -> String {
        let s = s.as_ref();
        let s = unindent(s);

        let regex = regex::Regex::new(r"\{-*\}").unwrap();

        s.lines()
            .map(|line| {
                let (style, line) = split_line(line, " $$");
                let line = regex.replace_all(&line, "").to_string();
                format!("{style}{line}\n", style = style.trim(), line = line)
            })
            .collect()
    }
}
