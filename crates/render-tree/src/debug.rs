use crate::stylesheet::WriteStyle;
use crate::Document;
use crate::{Node, PadItem};
use crate::{Style, Stylesheet};
use std::{fmt, io};
use termcolor::WriteColor;

struct DebugDocument<'a, C: WriteColor + 'a> {
    document: &'a Document,
    writer: &'a mut C,
    stylesheet: &'a Stylesheet,
    line_start: bool,
    nesting: Vec<&'static str>,
}

impl<'a, C: WriteColor + 'a> DebugDocument<'a, C> {
    fn write_document(mut self) -> io::Result<()> {
        let tree = match self.document.tree() {
            None => return Ok(()),
            Some(nodes) => nodes,
        };

        self.writer.reset()?;

        for item in tree.clone() {
            match item {
                Node::Text(string) => self.write_text(string)?,
                Node::OpenSection(section) => self.write_open_section(section)?,
                Node::CloseSection => self.write_close_section()?,
                Node::Newline => self.write_newline()?,
            }
        }

        write!(self.writer, "\n\n")?;

        Ok(())
    }

    fn write_text(&mut self, string: &str) -> io::Result<()> {
        if self.line_start {
            self.start_line()?;
            self.styled_write("|", "fg: black; weight: bold")?;
        }

        self.write(string)?;
        self.line_start = false;

        Ok(())
    }

    fn write_open_section(&mut self, section: &'static str) -> io::Result<()> {
        self.start_line()?;
        self.write("<")?;

        self.nesting.push(section);
        let style = self.stylesheet.get(&self.nesting[..]);

        self.styled_write(section, "fg: blue; weight: bold")?;

        if let Some(style) = style {
            if style.has_value() {
                self.write(" ")?;
                let debug_attributes = style.debug_attributes();
                let last = debug_attributes.len() - 1;

                for (i, (name, value)) in debug_attributes.iter().enumerate() {
                    self.styled_write(name, "fg: black; weight: bold")?;

                    if let Some(value) = value {
                        self.write("=")?;
                        self.styled_write(value, "fg: cyan; weight: dim")?;
                    }

                    if i != last {
                        self.write(" ")?;
                    }
                }

                self.styled_write(" §", style)?;
            }
        }

        self.writer.reset()?;
        write!(self.writer, ">")?;
        self.line_start = true;

        Ok(())
    }

    fn write_close_section(&mut self) -> io::Result<()> {
        let popped = self.nesting.pop().expect("unbalanced push/pop");
        self.start_line()?;
        write!(self.writer, "</")?;

        self.writer.set_style(&Style("fg: blue; weight: bold"))?;
        write!(self.writer, "{}", popped)?;
        self.writer.reset()?;
        write!(self.writer, ">")?;
        self.line_start = true;

        Ok(())
    }

    fn write_newline(&mut self) -> io::Result<()> {
        let writer = &mut self.writer;
        writer.reset()?;

        if self.line_start {
            write!(writer, "\n{}", PadItem("  ", self.nesting.len()))?;
        }

        write!(writer, "\\n",)?;
        self.line_start = true;

        Ok(())
    }

    fn start_line(&mut self) -> io::Result<()> {
        let pad = self.pad();
        write!(self.writer, "\n{}", pad)
    }

    fn styled_write(
        &mut self,
        value: impl fmt::Display,
        style: impl Into<Style>,
    ) -> io::Result<()> {
        self.writer.set_style(style)?;
        self.write(value)?;
        self.writer.reset()
    }

    fn write(&mut self, value: impl fmt::Display) -> io::Result<()> {
        write!(self.writer, "{}", value)
    }

    fn pad(&self) -> PadItem<&'static str> {
        PadItem(" ", self.nesting.len())
    }
}

impl Document {
    pub fn debug_write(
        &self,
        writer: &mut impl WriteColor,
        stylesheet: &Stylesheet,
    ) -> io::Result<()> {
        DebugDocument {
            document: self,
            writer,
            stylesheet,
            line_start: true,
            nesting: vec![],
        }.write_document()
    }
}
