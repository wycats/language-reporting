use std::fmt::Debug;
use std::path::PathBuf;

pub enum FileName {
    Virtual(PathBuf),
    Real(PathBuf),
    Verbatim(String),
}

pub struct Location {
    pub line: usize,
    pub column: usize,
}

pub trait ReportingSpan: Debug + Clone {
    fn file_name(&self) -> FileName;
    fn with_start(&self, start: usize) -> Self;
    fn with_end(&self, end: usize) -> Self;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

pub trait ReportingFiles: Debug {
    type InnerSpan: ReportingSpan;
    type FileId;

    fn byte_span(
        &self,
        file: &Self::FileId,
        from_index: usize,
        to_index: usize,
    ) -> Option<Self::InnerSpan>;

    fn byte_index(&self, file: &Self::FileId, line: usize, column: usize) -> Option<usize>;
    fn location(&self, file: &Self::FileId, byte_index: usize) -> Option<Location>;
    fn line_span(&self, file: &Self::FileId, lineno: usize) -> Option<Self::InnerSpan>;
    fn source(&self, span: &Self::InnerSpan) -> Option<&str>;
}
