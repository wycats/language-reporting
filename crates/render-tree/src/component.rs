use crate::{Document, Render};

pub trait BlockComponent: Sized {
    fn with<F: FnOnce(Document) -> Document>(
        component: Self,
        block: F,
    ) -> CurriedBlockComponent<Self, F> {
        CurriedBlockComponent { component, block }
    }

    fn append(self, block: impl FnOnce(Document) -> Document, document: Document) -> Document;
}

pub struct CurriedBlockComponent<B: BlockComponent, Block: FnOnce(Document) -> Document> {
    component: B,
    block: Block,
}

impl<B: BlockComponent, Block: FnOnce(Document) -> Document> Render
    for CurriedBlockComponent<B, Block>
{
    fn render(self, document: Document) -> Document {
        (self.component).append(self.block, document)
    }
}

// IterBlockComponent //

pub trait IterBlockComponent: Sized {
    type Item;

    fn with<F: FnMut(Self::Item, Document) -> Document>(
        component: Self,
        block: F,
    ) -> CurriedIterBlockComponent<Self, F> {
        CurriedIterBlockComponent { component, block }
    }

    fn append(
        self,
        block: impl FnMut(Self::Item, Document) -> Document,
        document: Document,
    ) -> Document;
}

pub struct CurriedIterBlockComponent<
    B: IterBlockComponent,
    Block: FnMut(B::Item, Document) -> Document,
> {
    component: B,
    block: Block,
}

impl<B: IterBlockComponent, Block: FnMut(B::Item, Document) -> Document> Render
    for CurriedIterBlockComponent<B, Block>
{
    fn render(self, document: Document) -> Document {
        (self.component).append(self.block, document)
    }
}

// OnceBlockComponent //

pub trait OnceBlockComponent: Sized {
    type Item;

    fn with<F: FnOnce(Self::Item, Document) -> Document>(
        component: Self,
        block: F,
    ) -> CurriedOnceBlockComponent<Self, F> {
        CurriedOnceBlockComponent { component, block }
    }

    fn append(
        self,
        block: impl FnOnce(Self::Item, Document) -> Document,
        document: Document,
    ) -> Document;
}

pub struct CurriedOnceBlockComponent<
    B: OnceBlockComponent,
    Block: FnOnce(B::Item, Document) -> Document,
> {
    component: B,
    block: Block,
}

impl<B: OnceBlockComponent, Block: FnOnce(B::Item, Document) -> Document> Render
    for CurriedOnceBlockComponent<B, Block>
{
    fn render(self, document: Document) -> Document {
        (self.component).append(self.block, document)
    }
}

// InlineComponent //

struct CurriedInlineComponent<T> {
    function: fn(T, Document) -> Document,
    data: T,
}

impl<T> Render for CurriedInlineComponent<T> {
    fn render(self, document: Document) -> Document {
        (self.function)(self.data, document)
    }
}

#[allow(non_snake_case)]
pub fn Component<T>(function: fn(T, Document) -> Document, data: T) -> impl Render {
    CurriedInlineComponent { function, data }
}

/// This trait defines a renderable entity with arguments. Types that implement
/// `RenderComponent` can be packaged up together with their arguments in a
/// `Component`, and the `Component` is renderable.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate render_tree;
/// extern crate termcolor;
/// use render_tree::{Document, Line, Render, RenderComponent};
/// use termcolor::StandardStream;
///
/// struct MessageContents {
///     code: usize,
///     header: String,
///     body: String,
/// }
///
/// fn Message(args: MessageContents, into: Document) -> Document {
///     into.add(tree! {
///         <Line as {
///             {args.code} ":" {args.header}
///         }>
///
///         <Line as {
///             {args.body}
///         }>
///     })
/// }
///
/// fn main() -> std::io::Result<()> {
///     let message = MessageContents {
///         code: 200,
///         header: "Hello world".to_string(),
///         body: "This is the body of the message".to_string()
///     };
///
///     let document = tree! { <Message args={message}> };
///
///     document.write()
/// }
/// ```
pub trait RenderComponent<'args> {
    type Args;

    fn render(&self, args: Self::Args, into: Document) -> Document;
}

pub struct OnceBlock<F: FnOnce(Document) -> Document>(pub F);

impl<F> Render for OnceBlock<F>
where
    F: FnOnce(Document) -> Document,
{
    fn render(self, into: Document) -> Document {
        (self.0)(into)
    }
}

#[cfg(test)]
mod tests {
    use crate::component::*;

    #[test]
    fn test_inline_component() -> ::std::io::Result<()> {
        struct Header {
            code: usize,
            message: &'static str,
        }

        impl Render for Header {
            fn render(self, document: Document) -> Document {
                document.add(tree! {
                    {self.code} {": "} {self.message}
                })
            }
        }

        let code = 1;
        let message = "Something went wrong";

        let document = tree! {
            <Header code={code} message={message}>
        };

        assert_eq!(document.to_string()?, "1: Something went wrong");

        Ok(())
    }

    #[test]
    fn test_block_component() -> ::std::io::Result<()> {
        struct Message {
            code: usize,
            message: &'static str,
            trailing: &'static str,
        }

        impl BlockComponent for Message {
            fn append(
                self,
                block: impl FnOnce(Document) -> Document,
                mut document: Document,
            ) -> Document {
                document = document.add(tree! {
                    {self.code} {": "} {self.message} {" "}
                });

                document = block(document);

                document = document.add(tree! {
                    {self.trailing}
                });

                document
            }
        }

        let code = 1;
        let message = "Something went wrong";

        let document = tree! {
            <Message code={code} message={message} trailing={" -- yikes!"} as {
                {"!!! It's really quite bad !!!"}
            }>
        };

        assert_eq!(
            document.to_string()?,
            "1: Something went wrong !!! It's really quite bad !!! -- yikes!"
        );

        Ok(())
    }

    #[test]
    fn test_once_block_component() -> ::std::io::Result<()> {
        struct Message {
            code: usize,
            message: Option<&'static str>,
            trailing: &'static str,
        }

        impl OnceBlockComponent for Message {
            type Item = String;

            fn append(
                self,
                block: impl FnOnce(String, crate::Document) -> crate::Document,
                mut document: crate::Document,
            ) -> crate::Document {
                document = document.add(tree! {
                    {self.code} {": "}
                });

                if let Some(message) = self.message {
                    document = block(message.to_string(), document);
                }

                document = document.add(tree! {
                    {" "} {self.trailing}
                });

                document
            }
        }

        let code = 1;
        let message = Some("Something went wrong");

        let document = tree! {
            <Message code={code} message={message} trailing={"-- yikes!"} as |message| {
                {message} {" "} {"!!! It's really quite bad !!!"}
            }>
        };

        assert_eq!(
            document.to_string()?,
            "1: Something went wrong !!! It's really quite bad !!! -- yikes!"
        );

        Ok(())
    }
}
