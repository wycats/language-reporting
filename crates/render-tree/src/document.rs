use crate::stylesheet::WriteStyle;
use crate::Stylesheet;
use crate::{Combine, Render};
use std::io;
use termcolor::{ColorChoice, StandardStream, WriteColor};

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    OpenSection(&'static str),
    CloseSection,
    Newline,
}

/// The `Document` is the root node in a render tree.
///
/// The [`tree!`] macro produces a `Document`, and you can also build
/// one manually.
///
/// ```
/// use render_tree::prelude::*;
/// use render_tree::Render;
///
/// fn main() -> std::io::Result<()> {
///     let document = Document::empty()
///         // You can add a `Line` to a document, with nested content
///         .add(Line(
///             // Strings implement `Render`
///             "Hello"
///         ))
///         .add(Line(
///             1.add(".").add(10)
///         ))
///         .add(Section("code", |doc|
///             doc.add("[E").add(1000).add("]")
///         ));
///
///     assert_eq!(document.to_string()?, "Hello\n1.10\n[E1000]");
///
///     Ok(())
/// }
/// ```
///
/// The above example is equivalent to this use of the [`tree!`] macro:
///
/// ```
/// #[macro_use]
/// extern crate render_tree;
/// use render_tree::prelude::*;
///
/// fn main() -> std::io::Result<()> {
///     let document = tree! {
///         <Line as { "Hello" }>
///         <Line as {
///             {1} "." {10}
///         }>
///         <Section name="code" as {
///             "[E" {1000} "]"
///         }>
///     };
///
///     assert_eq!(document.to_string()?, "Hello\n1.10\n[E1000]");
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Document {
    // Make the inner tree optional so it's free to create empty documents
    tree: Option<Vec<Node>>,
}

impl Document {
    pub fn empty() -> Document {
        Document { tree: None }
    }

    pub fn with(renderable: impl Render) -> Document {
        renderable.render(Document::empty())
    }

    pub(crate) fn tree(&self) -> Option<&[Node]> {
        match &self.tree {
            None => None,
            Some(vec) => Some(&vec[..]),
        }
    }

    fn initialize_tree(&mut self) -> &mut Vec<Node> {
        if self.tree.is_none() {
            self.tree = Some(vec![]);
        }

        match &mut self.tree {
            Some(value) => value,
            None => unreachable!(),
        }
    }

    pub fn add(self, renderable: impl Render) -> Document {
        renderable.render(self)
    }

    pub(crate) fn add_node(mut self, node: Node) -> Document {
        self.initialize_tree().push(node);
        self
    }

    pub(crate) fn extend_nodes(mut self, other: Vec<Node>) -> Document {
        if other.len() > 0 {
            let tree = self.initialize_tree();

            for item in other {
                tree.push(item)
            }
        }

        self
    }

    pub(crate) fn extend(self, fragment: Document) -> Document {
        match (&self.tree, &fragment.tree) {
            (Some(_), Some(_)) => self.extend_nodes(fragment.tree.unwrap()),
            (Some(_), None) => self,
            (None, Some(_)) => fragment,
            (None, None) => self,
        }
    }

    pub fn write(self) -> io::Result<()> {
        let mut writer = StandardStream::stdout(ColorChoice::Always);

        self.write_with(&mut writer, &Stylesheet::new())
    }

    pub fn to_string(self) -> io::Result<String> {
        let mut writer = ::termcolor::Buffer::no_color();
        let stylesheet = Stylesheet::new();

        self.write_with(&mut writer, &stylesheet)?;

        Ok(String::from_utf8_lossy(writer.as_slice()).into())
    }

    pub fn write_styled(self, stylesheet: &Stylesheet) -> io::Result<()> {
        let mut writer = StandardStream::stdout(ColorChoice::Always);

        self.write_with(&mut writer, stylesheet)
    }

    pub fn write_with(
        self,
        writer: &mut impl WriteColor,
        stylesheet: &Stylesheet,
    ) -> io::Result<()> {
        let mut nesting = vec![];

        writer.reset()?;

        let tree = match self.tree {
            None => return Ok(()),
            Some(nodes) => nodes,
        };

        for item in tree {
            match item {
                Node::Text(string) => {
                    if string.len() != 0 {
                        let style = stylesheet.get(&nesting);

                        match style {
                            None => writer.reset()?,
                            Some(style) => writer.set_style(&style)?,
                        }

                        write!(writer, "{}", string)?;
                    }
                }
                Node::OpenSection(section) => nesting.push(section),
                Node::CloseSection => {
                    nesting.pop().expect("unbalanced push/pop");
                }
                Node::Newline => {
                    writer.reset()?;
                    write!(writer, "\n")?;
                }
            }
        }

        Ok(())
    }
}

pub fn add<Left: Render, Right: Render>(left: Left, right: Right) -> Combine<Left, Right> {
    Combine { left, right }
}
