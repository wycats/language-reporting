use crate::diagnostic::Diagnostic;
use crate::{FileName, Label, LabelStyle, Location, ReportingFiles, ReportingSpan, Severity};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Header<'doc> {
    severity: Severity,
    code: Option<&'doc str>,
    message: &'doc str,
}

impl<'doc> Header<'doc> {
    pub(crate) fn new(diagnostic: &'doc Diagnostic<impl ReportingSpan>) -> Header<'doc> {
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

pub(crate) fn severity(diagnostic: &Diagnostic<impl ReportingSpan>) -> &'static str {
    match diagnostic.severity {
        Severity::Bug => "bug",
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Help => "help",
        Severity::Note => "note",
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct SourceLine<'doc, Files: ReportingFiles> {
    files: &'doc Files,
    label: &'doc Label<Files::Span>,
    config: &'doc dyn crate::Config,
}

impl<Files: ReportingFiles> SourceLine<'doc, Files> {
    pub(crate) fn new(
        files: &'doc Files,
        label: &'doc Label<Files::Span>,
        config: &'doc dyn crate::Config,
    ) -> SourceLine<'doc, Files> {
        SourceLine {
            files,
            label,
            config,
        }
    }

    pub(crate) fn location(&self) -> Location {
        let span = self.label.span;

        self.files
            .location(self.files.file_id(span), span.start())
            .expect("A valid location")
    }

    pub(crate) fn filename(&self) -> String {
        match &self.files.file_name(self.files.file_id(self.label.span)) {
            FileName::Virtual(name) => format!("<{}>", name.to_str().unwrap()),
            FileName::Real(name) => self.config.filename(name),
            FileName::Verbatim(name) => format!("{}", name),
        }
    }

    pub(crate) fn line_span(&self) -> Files::Span {
        let span = self.label.span;

        self.files
            .line_span(self.files.file_id(span), self.location().line)
            .expect("line_span")
    }

    pub(crate) fn line_number(&self) -> usize {
        self.location().line + 1
    }

    pub(crate) fn line_number_len(&self) -> usize {
        self.line_number().to_string().len()
    }

    // pub(crate) fn before_line_len(&self) -> usize {
    //     // TODO: Improve
    //     self.before_marked().len() + self.line_number().to_string().len()
    // }

    pub(crate) fn before_marked(&self) -> String {
        self.files
            .source(self.line_span().with_end(self.label.span.start()))
            .expect("line_prefix")
    }

    pub(crate) fn after_marked(&self) -> String {
        self.files
            .source(self.line_span().with_start(self.label.span.end()))
            .expect("line_suffix")
            .trim_end_matches(|ch| ch == '\r' || ch == '\n')
            .to_string()
    }

    pub(crate) fn marked(&self) -> String {
        self.files.source(self.label.span).expect("line_marked")
    }
}

#[derive(Clone)]
pub struct LabelledLine<'doc, Files: ReportingFiles> {
    source_line: SourceLine<'doc, Files>,
    label: &'doc Label<Files::Span>,
}

impl<Files: ReportingFiles> LabelledLine<'doc, Files> {
    pub(crate) fn new(
        source_line: SourceLine<'doc, Files>,
        label: &'doc Label<Files::Span>,
    ) -> LabelledLine<'doc, Files> {
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

    pub(crate) fn source_line(&self) -> &SourceLine<'doc, Files> {
        &self.source_line
    }
}
