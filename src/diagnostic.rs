use crate::{ReportingSpan, Severity};
use serde_derive::{Serialize, Deserialize};

/// A style for the label
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LabelStyle {
    /// The main focus of the diagnostic
    Primary,
    /// Supporting labels that may help to isolate the cause of the diagnostic
    Secondary,
}

/// A label describing an underlined region of code associated with a diagnostic
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label<Span: ReportingSpan> {
    /// The span we are going to include in the final snippet.
    pub span: Span,
    /// A message to provide some additional information for the underlined code.
    pub message: Option<String>,
    /// The style to use for the label.
    pub style: LabelStyle,
}

impl<Span: ReportingSpan> Label<Span> {
    pub fn new(span: Span, style: LabelStyle) -> Label<Span> {
        Label {
            span,
            message: None,
            style,
        }
    }

    pub fn new_primary(span: Span) -> Label<Span> {
        Label::new(span, LabelStyle::Primary)
    }

    pub fn new_secondary(span: Span) -> Label<Span> {
        Label::new(span, LabelStyle::Secondary)
    }

    pub fn with_message<S: Into<String>>(mut self, message: S) -> Label<Span> {
        self.message = Some(message.into());
        self
    }

    pub fn message(&self) -> &Option<String> {
        &self.message
    }
}

/// Represents a diagnostic message and associated child messages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Diagnostic<Span: ReportingSpan> {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic
    pub message: String,
    /// The labelled spans marking the regions of code that cause this
    /// diagnostic to be raised
    pub labels: Vec<Label<Span>>,
}

impl<Span: ReportingSpan> Diagnostic<Span> {
    pub fn new<S: Into<String>>(severity: Severity, message: S) -> Diagnostic<Span> {
        Diagnostic {
            severity,
            code: None,
            message: message.into(),
            labels: Vec::new(),
        }
    }

    pub fn new_bug<S: Into<String>>(message: S) -> Diagnostic<Span> {
        Diagnostic::new(Severity::Bug, message)
    }

    pub fn new_error<S: Into<String>>(message: S) -> Diagnostic<Span> {
        Diagnostic::new(Severity::Error, message)
    }

    pub fn new_warning<S: Into<String>>(message: S) -> Diagnostic<Span> {
        Diagnostic::new(Severity::Warning, message)
    }

    pub fn new_note<S: Into<String>>(message: S) -> Diagnostic<Span> {
        Diagnostic::new(Severity::Note, message)
    }

    pub fn new_help<S: Into<String>>(message: S) -> Diagnostic<Span> {
        Diagnostic::new(Severity::Help, message)
    }

    pub fn with_code<S: Into<String>>(mut self, code: S) -> Diagnostic<Span> {
        self.code = Some(code.into());
        self
    }

    pub fn with_label(mut self, label: Label<Span>) -> Diagnostic<Span> {
        self.labels.push(label);
        self
    }

    pub fn with_labels<Labels: IntoIterator<Item = Label<Span>>>(
        mut self,
        labels: Labels,
    ) -> Diagnostic<Span> {
        self.labels.extend(labels);
        self
    }
}
