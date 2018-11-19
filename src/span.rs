use derive_new::new;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FileName {
    Virtual(PathBuf),
    Real(PathBuf),
    Verbatim(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, new)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

pub trait ReportingSpan: Debug + Copy {
    fn with_start(&self, start: usize) -> Self;
    fn with_end(&self, end: usize) -> Self;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

pub trait ReportingFiles: Debug + Clone {
    type Span: ReportingSpan;
    type FileId: Copy;

    fn byte_span(
        &self,
        file: Self::FileId,
        from_index: usize,
        to_index: usize,
    ) -> Option<Self::Span>;

    fn file_id(&self, span: Self::Span) -> Self::FileId;
    fn file_name(&self, file: Self::FileId) -> FileName;
    fn byte_index(&self, file: Self::FileId, line: usize, column: usize) -> Option<usize>;
    fn location(&self, file: Self::FileId, byte_index: usize) -> Option<Location>;
    fn line_span(&self, file: Self::FileId, lineno: usize) -> Option<Self::Span>;
    fn file_source(&self, span: Self::FileId) -> Option<&str>;
    fn source(&self, span: Self::Span) -> Option<&str>;
}
