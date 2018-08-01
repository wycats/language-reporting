use super::{Document, Node};

/// The Render trait defines a type that can be added to a Document.
/// It is defined for `Node`, `String`, `&str`, and `Document`.alloc
///
/// It is also defined for `Option<T>` where `T` is `Render`, as well
/// as `&T` where `T` is both `Render` and `Clone`.
///
/// Generally speaking, if you need to make a type `Render`, and it's
/// not one of your types, you can ergonomically make a newtype wrapper
/// for it.
///
/// For example, if you want to render `std::time::Duration`:
///
/// ```
/// #[macro_use]
/// extern crate render_tree;
/// extern crate termcolor;
/// use render_tree::{Render, Document, Line, RenderComponent};
/// use std::time::Duration;
/// use termcolor::StandardStream;
///
/// struct RenderDuration(Duration);
///
/// impl Render for RenderDuration {
///     fn render(self, into: Document) -> Document {
///         into.add(format!("{} seconds and {} nanos", self.0.as_secs(), self.0.subsec_nanos()))
///     }
/// }
///
/// struct MessageContents {
///     code: usize,
///     header: String,
///     body: String,
///     duration: Duration,
/// }
///
/// fn message(args: MessageContents, into: Document) -> Document {
///     into.render(tree! {
///         <Line as {
///             {args.code} ":" {args.header} "for" {RenderDuration(args.duration)}
///         }>
///
///         <Line as {
///             {args.body}
///         }>
///     })
/// }
///
/// fn main() -> std::io::Result<()> {
///     let contents = MessageContents {
///         code: 200,
///         header: "Hello world".to_string(),
///         body: "This is the body of the message".to_string(),
///         duration: Duration::new(100, 1_000_000)
///     };
///
///     let document = tree! { <message args={contents}> };
///
///     document.write()
/// }
/// ```
pub trait Render: Sized {
    /// Produce a new Document with `self` added to the `into` Document.
    fn render(self, into: Document) -> Document;

    fn into_fragment(self) -> Document {
        self.render(Document::empty())
    }

    fn add<Right: Render>(self, other: Right) -> Combine<Self, Right> {
        Combine {
            left: self,
            right: other,
        }
    }
}

pub struct Combine<Left: Render, Right: Render> {
    pub(crate) left: Left,
    pub(crate) right: Right,
}

impl<Left: Render, Right: Render> Render for Combine<Left, Right> {
    fn render(self, into: Document) -> Document {
        into.add(self.left).add(self.right)
    }
}

/// A node is rendered by adding itself to the document
impl Render for Node {
    fn render(self, document: Document) -> Document {
        document.add_node(self)
    }
}

/// A Document is rendered by extending its nodes onto the original
/// document.
impl Render for Document {
    fn render(self, into: Document) -> Document {
        into.extend(self)
    }
}

// /// An Option<impl Render> is rendered by doing nothing if None or
// /// rendering the inner value if Some.
// impl<T> Render for Option<T>
// where
//     T: Render,
// {
//     fn render(self, document: Document) -> Document {
//         match self {
//             None => document,
//             Some(item) => item.render(document),
//         }
//     }
// }

struct IfSome<'item, T: 'item, R: Render, F: Fn(&T) -> R + 'item> {
    option: &'item Option<T>,
    callback: F,
}

impl<'item, T, R, F> Render for IfSome<'item, T, R, F>
where
    T: 'item,
    R: Render,
    F: Fn(&T) -> R,
{
    fn render(self, mut into: Document) -> Document {
        if let Some(inner) = self.option {
            into = into.add((self.callback)(inner));
        }

        into
    }
}

#[allow(non_snake_case)]
pub fn IfSome<'item, T: 'item, R: Render + 'item>(
    option: &'item Option<T>,
    callback: impl Fn(&T) -> R + 'item,
) -> impl Render + 'item {
    IfSome { option, callback }
}

struct SomeValue<'item, T: 'item> {
    option: &'item Option<T>,
}

impl<'item, T> Render for SomeValue<'item, T>
where
    T: Render + Clone + 'item,
{
    fn render(self, mut into: Document) -> Document {
        if let Some(inner) = self.option {
            into = inner.clone().render(into)
        }

        into
    }
}

#[allow(non_snake_case)]
pub fn SomeValue<'item, R: Render + Clone>(option: &'item Option<R>) -> impl Render + 'item {
    SomeValue { option }
}

pub struct Empty;

impl Render for Empty {
    fn render(self, document: Document) -> Document {
        document
    }
}

impl<T: ::std::fmt::Display> Render for T {
    fn render(self, document: Document) -> Document {
        document.add(Node::Text(self.to_string()))
    }
}
