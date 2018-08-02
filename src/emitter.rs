use codespan::CodeMap;
use crate::components;
use crate::diagnostic::Diagnostic;
use log;
use render_tree::{Component, Render, Stylesheet};
use std::path::Path;
use std::{fmt, io};
use termcolor::WriteColor;

pub fn emit<'doc, W>(
    writer: W,
    codemap: &'doc CodeMap,
    diagnostic: &'doc Diagnostic,
    config: &'doc dyn Config,
) -> io::Result<()>
where
    W: WriteColor,
{
    DiagnosticWriter { writer }.emit(DiagnosticData {
        codemap,
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
    fn emit<'doc>(mut self, data: DiagnosticData<'doc>) -> io::Result<()> {
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

        if log_enabled!(log::Level::Debug) {
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
pub(crate) struct DiagnosticData<'doc> {
    pub(crate) codemap: &'doc CodeMap,
    pub(crate) diagnostic: &'doc Diagnostic,
    pub(crate) config: &'doc dyn Config,
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
    use codespan::*;
    use crate::diagnostic::{Diagnostic, Label};
    use crate::termcolor::Buffer;
    use crate::Severity;
    use regex;
    use render_tree::stylesheet::ColorAccumulator;
    use unindent::unindent;

    fn emit_with_writer<W: WriteColor>(mut writer: W) -> W {
        let mut code_map = CodeMap::new();

        let source = unindent(
            r##"
                (define test 123)
                (+ test "")
                ()
            "##,
        );

        let file_map = code_map.add_filemap("test".into(), source.to_string());

        let str_start = file_map.byte_index(1.into(), 8.into()).unwrap();
        let error = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
            .with_label(
                Label::new_primary(Span::from_offset(str_start, 2.into()))
                    .with_message("Expected integer but got string"),
            ).with_label(
                Label::new_secondary(Span::from_offset(str_start, 2.into()))
                    .with_message("Expected integer but got string"),
            ).with_code("E0001");

        let line_start = file_map.byte_index(1.into(), 0.into()).unwrap();
        let warning = Diagnostic::new(
            Severity::Warning,
            "`+` function has no effect unless its result is used",
        ).with_label(Label::new_primary(Span::from_offset(line_start, 11.into())));

        let diagnostics = [error, warning];

        for diagnostic in &diagnostics {
            emit(&mut writer, &code_map, &diagnostic, &super::DefaultConfig).unwrap();
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
                    - <test>:2:9
                    2 | (+ test "")
                      |         ^^ Expected integer but got string
                    - <test>:2:9
                    2 | (+ test "")
                      |         -- Expected integer but got string
                    warning: `+` function has no effect unless its result is used
                    - <test>:2:1
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
                                        $$- <test>:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Red}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Red}^^ Expected integer but got string{/}
                                        $$- <test>:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Cyan}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Cyan}-- Expected integer but got string{/}
                {fg:Yellow bold bright} $$warning{bold bright}: `+` function has no effect unless its result is used{/}
                                        $$- <test>:2:1
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
            }).collect()
    }
}
