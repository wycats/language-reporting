use super::{Match, Node, Segment};
use crate::Style;
use std::fmt;

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Segment::Name(s) => write!(f, "{}", s),
            Segment::Glob => write!(f, "**"),
            Segment::Star => write!(f, "*"),
            Segment::Root => write!(f, "ε"),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

pub(super) struct NodeDetails<'a> {
    segment: Segment,
    style: &'a Option<Style>,
}

impl<'a> NodeDetails<'a> {
    pub(super) fn new(segment: Segment, style: &'a Option<Style>) -> NodeDetails<'a> {
        NodeDetails { segment, style }
    }
}

impl<'a> fmt::Display for NodeDetails<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} style=", self.segment)?;

        match &self.style {
            None => write!(f, "None")?,
            Some(style) => write!(f, "{}", style)?,
        }

        Ok(())
    }
}

pub(super) struct DisplayStyle<'a>(pub(super) &'a Option<Style>);

impl<'a> fmt::Display for DisplayStyle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "None"),
            Some(style) => write!(f, "{}", style),
        }
    }
}

impl<'a> fmt::Display for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.glob.is_none()
            && self.star.is_none()
            && self.skipped_glob.is_none()
            && self.literal.is_none()
        {
            write!(f, "None")
        } else {
            write!(f, "[")?;

            let mut wrote_first = false;

            let mut comma = |f: &mut fmt::Formatter| -> fmt::Result {
                if wrote_first {
                    write!(f, ", ")
                } else {
                    wrote_first = true;
                    Ok(())
                }
            };

            if let Some(glob) = self.glob {
                comma(f)?;
                write!(f, "{}", glob.segment)?;
            }

            if let Some(star) = self.star {
                comma(f)?;
                write!(f, "{}", star.segment)?;
            }

            if let Some(skipped_glob) = self.skipped_glob {
                comma(f)?;
                write!(f, "skipped glob: {}", skipped_glob.segment)?;
            }

            if let Some(literal) = self.literal {
                comma(f)?;
                write!(f, "next: {}", literal.segment)?;
            }

            write!(f, "]")
        }
    }
}
