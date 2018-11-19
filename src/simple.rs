#[derive(Debug, Clone)]
pub struct SimpleFile {
    name: String,
    contents: String,
}

#[derive(Debug, Clone, Default)]
pub struct SimpleReportingFiles {
    files: Vec<SimpleFile>,
}

impl SimpleReportingFiles {
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) -> usize {
        self.files.push(SimpleFile {
            name: name.into(),
            contents: value.into(),
        });

        self.files.len() - 1
    }
}

impl crate::ReportingFiles for SimpleReportingFiles {
    type Span = SimpleSpan;
    type FileId = usize;

    fn file_id(&self, span: SimpleSpan) -> usize {
        span.file_id
    }

    fn file_name(&self, id: usize) -> crate::FileName {
        crate::FileName::Verbatim(self.files[id].name.clone())
    }

    fn byte_span(&self, _file: usize, _from_index: usize, _to_index: usize) -> Option<Self::Span> {
        unimplemented!()
    }

    fn byte_index(&self, file: usize, line: usize, column: usize) -> Option<usize> {
        let source = &self.files[file].contents;
        let mut seen_lines = 0;
        let mut seen_bytes = 0;

        for (pos, _) in source.match_indices('\n') {
            if seen_lines == line {
                return Some(seen_bytes + column);
            } else {
                seen_lines += 1;
                seen_bytes = pos + 1;
            }
        }

        None
    }

    fn location(&self, file: usize, index: usize) -> Option<crate::Location> {
        let source = &self.files[file].contents;
        let mut seen_lines = 0;
        let mut seen_bytes = 0;

        for (pos, _) in source.match_indices('\n') {
            if pos > index {
                return Some(crate::Location::new(seen_lines, index - seen_bytes));
            } else {
                seen_lines += 1;
                seen_bytes = pos;
            }
        }

        None
    }

    fn line_span(&self, file: usize, line: usize) -> Option<Self::Span> {
        let source = &self.files[file].contents;
        let mut seen_lines = 0;
        let mut seen_bytes = 0;

        for (pos, _) in source.match_indices('\n') {
            if seen_lines == line {
                return Some(SimpleSpan::new(file, seen_bytes, pos));
            } else {
                seen_lines += 1;
                seen_bytes = pos + 1;
            }
        }

        None
    }

    fn source(&self, span: SimpleSpan) -> Option<String> {
        let source = &self.files[span.file_id].contents;

        Some(source[span.start..span.end].to_string())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleSpan {
    file_id: usize,
    start: usize,
    end: usize,
}

impl SimpleSpan {
    pub fn new(file_id: usize, start: usize, end: usize) -> SimpleSpan {
        assert!(
            end >= start,
            "SimpleSpan {} must be bigger than {}",
            end,
            start
        );

        SimpleSpan {
            file_id,
            start,
            end,
        }
    }
}

impl crate::ReportingSpan for SimpleSpan {
    fn with_start(&self, start: usize) -> Self {
        SimpleSpan::new(self.file_id, start, self.end)
    }

    fn with_end(&self, end: usize) -> Self {
        SimpleSpan::new(self.file_id, self.start, end)
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
