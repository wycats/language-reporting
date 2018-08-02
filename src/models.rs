use codespan::{ByteSpan, ColumnIndex, FileMap, FileName, LineIndex, LineNumber};
use crate::diagnostic::Diagnostic;
use crate::{Label, LabelStyle, Severity};
use std::path::PathBuf;

pub(crate) struct Message<'doc> {
    message: &'doc Option<String>,
}

impl<'doc> Message<'doc> {
    pub(crate) fn new(message: &'doc Option<String>) -> Message<'doc> {
        Message { message }
    }

    pub(crate) fn message(&self) -> &Option<String> {
        &self.message
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Header<'doc> {
    severity: Severity,
    code: Option<&'doc str>,
    message: &'doc str,
}

impl<'doc> Header<'doc> {
    pub(crate) fn new(diagnostic: &'doc Diagnostic) -> Header<'doc> {
        Header {
            severity: diagnostic.severity,
            code: diagnostic.code.as_ref().map(|c| &c[..]),
            message: &diagnostic.message,
        }
    }

    pub(crate) fn severity(&self) -> &'static str {
        match self.severity {
            Severity::Bug => "bug",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Help => "help",
            Severity::Note => "note",
        }
    }

    pub(crate) fn code(&self) -> &Option<&'doc str> {
        &self.code
    }

    pub(crate) fn message(&self) -> String {
        self.message.to_string()
    }
}

pub(crate) fn severity(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic.severity {
        Severity::Bug => "bug",
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Help => "help",
        Severity::Note => "note",
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct SourceLine<'doc> {
    file: &'doc FileMap,
    label: &'doc Label,
}

impl<'doc> SourceLine<'doc> {
    pub(crate) fn new(
        file: &'doc FileMap,
        label: &'doc Label,
        config: &'doc dyn crate::Config,
    ) -> SourceLine<'doc> {
        SourceLine { file, label }
    }

    pub(crate) fn location(&self) -> (LineIndex, ColumnIndex) {
        self.file
            .location(self.label.span.start())
            .expect("location")
    }

    pub(crate) fn filename(&self) -> &'doc FileName {
        self.file.name()
    }

    pub(crate) fn line_span(&self) -> ByteSpan {
        self.file.line_span(self.location().0).expect("line_span")
    }

    pub(crate) fn line_number(&self) -> LineNumber {
        self.location().0.number()
    }

    pub(crate) fn line_number_len(&self) -> usize {
        self.line_number().to_string().len()
    }

    // pub(crate) fn before_line_len(&self) -> usize {
    //     // TODO: Improve
    //     self.before_marked().len() + self.line_number().to_string().len()
    // }

    pub(crate) fn before_marked(&self) -> &'doc str {
        self.file
            .src_slice(self.line_span().with_end(self.label.span.start()))
            .expect("line_prefix")
    }

    pub(crate) fn after_marked(&self) -> &'doc str {
        self.file
            .src_slice(self.line_span().with_start(self.label.span.end()))
            .expect("line_suffix")
            .trim_right_matches(|ch| ch == '\r' || ch == '\n')
    }

    pub(crate) fn marked(&self) -> &'doc str {
        self.file.src_slice(self.label.span).expect("line_marked")
    }
}

#[derive(Copy, Clone)]
pub struct LabelledLine<'doc> {
    source_line: SourceLine<'doc>,
    label: &'doc Label,
}

impl<'doc> LabelledLine<'doc> {
    pub(crate) fn new(source_line: SourceLine<'doc>, label: &'doc Label) -> LabelledLine<'doc> {
        LabelledLine { source_line, label }
    }

    pub(crate) fn mark(&self) -> &'static str {
        match self.label.style {
            LabelStyle::Primary => "^",
            LabelStyle::Secondary => "-",
        }
    }

    pub(crate) fn style(&self) -> &'static str {
        match self.label.style {
            LabelStyle::Primary => "primary",
            LabelStyle::Secondary => "secondary",
        }
    }

    pub(crate) fn message(&self) -> &Option<String> {
        self.label.message()
    }

    pub(crate) fn source_line(&self) -> &SourceLine<'doc> {
        &self.source_line
    }
}
