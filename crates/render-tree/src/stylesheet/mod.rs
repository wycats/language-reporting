mod accumulator;
mod color;
mod format;
mod style;

use self::format::{DisplayStyle, NodeDetails};
use crate::utils::CommaArray;
use crate::PadItem;
use itertools::Itertools;
use log::*;
use std::collections::HashMap;

pub use self::accumulator::ColorAccumulator;
pub use self::color::Color;
pub use self::style::{Style, WriteStyle};

pub struct Selector {
    segments: Vec<Segment>,
}

impl Selector {
    pub fn new() -> Selector {
        Selector { segments: vec![] }
    }

    pub fn glob() -> GlobSelector {
        Selector::new().add_glob()
    }

    pub fn star() -> Selector {
        Selector::new().add_star()
    }

    pub fn name(name: &'static str) -> Selector {
        Selector::new().add(name)
    }

    pub fn add_glob(self) -> GlobSelector {
        let mut segments = self.segments;
        segments.push(Segment::Glob);
        GlobSelector { segments }
    }

    pub fn add_star(mut self) -> Selector {
        self.segments.push(Segment::Star);
        self
    }

    pub fn add(mut self, segment: &'static str) -> Selector {
        self.segments.push(Segment::Name(segment));
        self
    }
}

/// This type statically prevents appending a glob right after another glob,
/// which is illegal. It shares the `add_star` and `add` methods with
/// `Selector`, but does not have an `add_glob` method.
pub struct GlobSelector {
    segments: Vec<Segment>,
}

impl GlobSelector {
    pub fn add_star(self) -> Selector {
        let mut segments = self.segments;
        segments.push(Segment::Star);
        Selector { segments }
    }

    pub fn add(self, segment: &'static str) -> Selector {
        let mut segments = self.segments;
        segments.push(Segment::Name(segment));
        Selector { segments }
    }
}

impl IntoIterator for Selector {
    type Item = Segment;
    type IntoIter = ::std::vec::IntoIter<Segment>;

    fn into_iter(self) -> ::std::vec::IntoIter<Segment> {
        self.segments.into_iter()
    }
}

impl IntoIterator for GlobSelector {
    type Item = Segment;
    type IntoIter = ::std::vec::IntoIter<Segment>;

    fn into_iter(self) -> ::std::vec::IntoIter<Segment> {
        self.segments.into_iter()
    }
}

impl From<&'static str> for Selector {
    fn from(from: &'static str) -> Selector {
        let segments = from.split(' ');
        let segments = segments.map(|part| part.into());

        Selector {
            segments: segments.collect(),
        }
    }
}

/// A Segment is one of:
///
/// - Root: The root node
/// - Star: `*`, matches exactly one section names
/// - Glob: `**`, matches zero or more section names
/// - Name: A named segment, matches a section name that exactly matches the name
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Segment {
    Root,
    Star,
    Glob,
    Name(&'static str),
}

impl From<&'static str> for Segment {
    fn from(from: &'static str) -> Segment {
        if from == "**" {
            Segment::Glob
        } else if from == "*" {
            Segment::Star
        } else {
            Segment::Name(from)
        }
    }
}

/// A Node represents a segment, child segments, and an optional associated style.
#[derive(Debug)]
struct Node {
    segment: Segment,
    children: HashMap<Segment, Node>,
    declarations: Option<Style>,
}

impl Node {
    fn new(segment: Segment) -> Node {
        Node {
            segment,
            children: HashMap::new(),
            declarations: None,
        }
    }

    fn display<'a>(&'a self) -> NodeDetails<'a> {
        NodeDetails::new(self.segment, &self.declarations)
    }

    /// Return a terminal node relative to the current node. If the current
    /// node has no children, it's the terminal node. Otherwise, if the
    /// current node has a glob child, that child is the terminal node.
    ///
    /// Otherwise, this node is not a terminal node.
    fn terminal(&self) -> Option<&Node> {
        match self.children.get(&Segment::Glob) {
            None => if self.children.is_empty() {
                return Some(self);
            } else {
                return None;
            },
            Some(glob) => return Some(glob),
        };
    }

    /// Add nodes for the segment path, and associate it with the provided style.
    fn add(&mut self, selector: impl IntoIterator<Item = Segment>, declarations: impl Into<Style>) {
        let mut path = selector.into_iter();

        match path.next() {
            None => {
                self.declarations = Some(declarations.into());
            }
            Some(name) => self
                .children
                .entry(name)
                .or_insert(Node::new(name))
                .add(path, declarations),
        }
    }

    /// Find a style for a section path. The resulting style is the merged result of all
    /// matches, with literals taking precedence over stars and stars taking precedence
    /// over globs.
    ///
    /// Earlier nodes take precedence over later nodes, so:
    ///
    /// `header *` takes precedence over `* code` for the section path `["header", "code"]`.
    ///
    /// Styles are merged per attribute, so the style attributes for a lower-precedence rule
    /// will appear in the merged style as long as they are not overridden by a
    /// higher-precedence rule.
    fn find<'a>(&self, names: &[&'static str], debug_nesting: usize) -> Option<Style> {
        trace!(
            "{}In {}, finding {:?} (children={})",
            PadItem("  ", debug_nesting),
            self,
            names.join(" "),
            CommaArray(self.children.keys().map(|k| k.to_string()).collect())
        );

        let next_name = match names.first() {
            None => {
                let terminal = self.terminal()?;

                trace!(
                    "{}Matched terminal {}",
                    PadItem("  ", debug_nesting),
                    terminal.display()
                );

                return terminal.declarations.clone();
            }

            Some(next_name) => next_name,
        };

        let matches = self.find_match(next_name);

        trace!("{}Matches: {}", PadItem("  ", debug_nesting), matches);

        // Accumulate styles from matches, in order of precedence.
        let mut style: Option<Style> = None;

        // A glob match means that a child node of the current node was a glob. Since
        // globs match zero or more segments, if a node has a glob child, it will
        // always match.
        if let Some(glob) = matches.glob {
            style = union(style, glob.find(&names[1..], debug_nesting + 1));
            trace!(
                "{}matched glob={}",
                PadItem("  ", debug_nesting),
                DisplayStyle(&style)
            );
        }

        // A star matches exactly one segment.
        if let Some(star) = matches.star {
            style = union(style, star.find(&names[1..], debug_nesting + 1));
            trace!(
                "{}matched star={}",
                PadItem("  ", debug_nesting),
                DisplayStyle(&style)
            );
        }

        if let Some(skipped_glob) = matches.skipped_glob {
            style = union(style, skipped_glob.find(&names[1..], debug_nesting + 1));
            trace!(
                "{}matched skipped_glob={}",
                PadItem("  ", debug_nesting),
                DisplayStyle(&style)
            );
        }

        if let Some(literal) = matches.literal {
            style = union(style, literal.find(&names[1..], debug_nesting + 1));
            trace!(
                "{}matched literal={}",
                PadItem("  ", debug_nesting),
                DisplayStyle(&style)
            );
        }

        style
    }

    /// Find a match in the current node for a section name.
    ///
    /// - If the current node is a glob, the current node is a match, since a
    ///   glob node can absorb arbitrarily many section names.alloc
    /// - If the current node has a glob child, it's a match. These two
    ///   conditions cannot occur at the same time, since a glob cannot
    ///   immediately follow a glob.
    /// - If the current node has a glob child, and it's immediately
    ///   followed by a literal node that matches the section, that
    ///   node is a match.
    /// - If the current node has a star child, it's a match
    ///
    /// The matches are applied in precedence order.
    fn find_match<'a>(&'a self, name: &'static str) -> Match<'a> {
        let glob;

        let mut skipped_glob = None;
        let star = self.children.get(&Segment::Star);
        let literal = self.children.get(&Segment::Name(name));

        // A glob always matches itself
        if self.segment == Segment::Glob {
            glob = Some(self);
        } else {
            glob = self.children.get(&Segment::Glob);

            if let Some(glob) = glob {
                skipped_glob = glob.children.get(&Segment::Name(name));
            }
        }

        Match {
            glob,
            star,
            skipped_glob,
            literal,
        }
    }
}

fn union(left: Option<Style>, right: Option<Style>) -> Option<Style> {
    match (left, right) {
        (None, None) => None,
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (Some(left), Some(right)) => Some(left.union(right)),
    }
}

struct Match<'a> {
    glob: Option<&'a Node>,
    star: Option<&'a Node>,
    skipped_glob: Option<&'a Node>,
    literal: Option<&'a Node>,
}

#[derive(Debug)]
pub struct Stylesheet {
    styles: Node,
}

impl Stylesheet {
    /// Construct a new stylesheet
    pub fn new() -> Stylesheet {
        Stylesheet {
            styles: Node::new(Segment::Root),
        }
    }

    /// Add a segment to the stylesheet.
    ///
    /// Using style strings:
    ///
    /// ```
    /// # use render_tree::{Stylesheet, Style};
    ///
    /// let stylesheet = Stylesheet::new()
    ///     .add("message header * code", "weight: bold; fg: red");
    ///
    /// assert_eq!(stylesheet.get(&["message", "header", "error", "code"]),
    ///     Some(Style("weight: bold; fg: red")))
    /// ```
    ///
    /// Using typed styles:
    ///
    /// ```
    /// # use render_tree::{Color, Style, Stylesheet};
    /// #
    /// let stylesheet = Stylesheet::new()
    ///     .add("message header * code", Style::new().bold().fg(Color::Red));
    ///
    /// assert_eq!(stylesheet.get(&["message", "header", "error", "code"]),
    ///     Some(Style("weight: bold; fg: red")))
    /// ```
    pub fn add(mut self, name: impl Into<Selector>, declarations: impl Into<Style>) -> Stylesheet {
        self.styles.add(name.into(), declarations);

        self
    }

    /// Get the style associated with a nesting.
    ///
    /// ```
    /// # use render_tree::{Stylesheet, Style};
    ///
    /// let stylesheet = Stylesheet::new()
    ///     .add("message ** code", "fg: blue")
    ///     .add("message header * code", "weight: bold; fg: red");
    ///
    /// let style = stylesheet.get(&["message", "header", "error", "code"]);
    /// ```
    pub fn get(&self, names: &[&'static str]) -> Option<Style> {
        if log_enabled!(::log::Level::Trace) {
            println!("\n");
        }

        trace!("Searching for `{}`", names.iter().join(" "));
        let style = self.styles.find(names, 0);

        match &style {
            None => trace!("No style found"),
            Some(style) => trace!("Found {}", style),
        }

        style
    }
}

#[cfg(test)]
mod tests {
    use super::style::Style;
    use crate::{Color, Stylesheet};
    use pretty_env_logger;

    fn init_logger() {
        pretty_env_logger::try_init().ok();
    }

    #[test]
    fn test_basic_lookup() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message header error code", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_basic_with_typed_style() {
        init_logger();

        let stylesheet = Stylesheet::new().add(
            "message header error code",
            Style::new().bold().fg(Color::Red),
        );

        assert_eq!(
            stylesheet.get(&["message", "header", "error", "code"]),
            Some(Style("weight: bold; fg: red"))
        )
    }

    #[test]
    fn test_star() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message header * code", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_star_with_typed_style() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message header * code", Style::new().bold().fg(Color::Red));

        assert_eq!(
            stylesheet.get(&["message", "header", "error", "code"]),
            Some(Style("weight: bold; fg: red"))
        )
    }

    #[test]
    fn test_glob() {
        init_logger();

        let stylesheet = Stylesheet::new().add("message ** code", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_glob_with_typed_style() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message ** code", Style::new().nounderline().fg(Color::Red));

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_glob_matches_no_segments() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message ** header error code", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_glob_matches_no_segments_with_typed_style() {
        init_logger();

        let stylesheet = Stylesheet::new().add(
            "message ** header error code",
            Style::new().nounderline().fg(Color::Red),
        );

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_trailing_glob_is_terminal() {
        init_logger();

        let stylesheet = Stylesheet::new().add(
            "message header error **",
            Style::new().nounderline().fg(Color::Red),
        );

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style("fg: red; underline: false")))
    }

    #[test]
    fn test_trailing_glob_is_terminal_with_typed_styles() {
        init_logger();

        let stylesheet = Stylesheet::new().add(
            "message header error **",
            Style::new().nounderline().fg(Color::Red),
        );

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style::new().fg(Color::Red).nounderline()))
    }

    #[test]
    fn test_trailing_glob_is_terminal_and_matches_nothing() {
        init_logger();

        let stylesheet =
            Stylesheet::new().add("message header error code **", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style::new().fg(Color::Red).nounderline()))
    }

    #[test]
    fn test_trailing_glob_is_terminal_and_matches_nothing_with_typed_style() {
        init_logger();

        let stylesheet = Stylesheet::new().add(
            "message header error code **",
            Style::new().nounderline().fg(Color::Red),
        );

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(style, Some(Style::new().fg(Color::Red).nounderline()))
    }

    #[test]
    fn test_priority() {
        init_logger();

        let stylesheet = Stylesheet::new()
            .add("message ** code", "fg: blue; weight: bold")
            .add("message header * code", "underline: true; bg: black")
            .add("message header error code", "fg: red; underline: false");

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(
            style,
            Some(
                Style::new()
                    .fg(Color::Red)
                    .bg(Color::Black)
                    .nounderline()
                    .bold()
            )
        )
    }

    #[test]
    fn test_priority_with_typed_style() {
        init_logger();

        let stylesheet = Stylesheet::new()
            .add("message ** code", Style::new().fg(Color::Blue).bold())
            .add(
                "message header * code",
                Style::new().underline().bg(Color::Black),
            ).add(
                "message header error code",
                Style::new().fg(Color::Red).nounderline(),
            );

        let style = stylesheet.get(&["message", "header", "error", "code"]);

        assert_eq!(
            style,
            Some(
                Style::new()
                    .fg(Color::Red)
                    .bg(Color::Black)
                    .nounderline()
                    .bold()
            )
        )
    }
}
