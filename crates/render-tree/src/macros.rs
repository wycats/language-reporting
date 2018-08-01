/// This macro builds a [`Document`] using nested syntax.
///
/// # Inline values using `{...}` syntax
///
/// You can insert any [`Render`] value into a document using `{...}` syntax.
///
/// ```
/// # #[macro_use]
/// # extern crate render_tree;
/// # fn main() -> ::std::io::Result<()> {
/// use render_tree::prelude::*;
///
/// let hello = "hello";
/// let world = format!("world");
/// let title = ". The answer is ";
/// let answer = 42;
///
/// let document = tree! {
///     {hello} {" "} {world} {". The answer is "} {answer}
/// };
///
/// assert_eq!(document.to_string()?, "hello world. The answer is 42");
/// #
/// # Ok(())
/// # }
/// ```
///
/// Built-in types that implement render include:
///
/// - Anything that implements `Display` (including String, &str, the number types, etc.).
///   The text value is inserted into the document.
/// - Other [`Document`]s, which are concatenated onto the document.
/// - A [`SomeValue`] adapter that takes an `Option<impl Renderable>` and inserts its inner
///   value if present.
/// - An [`Empty`] value that adds nothing to the document.
///
/// # Inline Components
///
/// You can create components to encapsulate some logic:
///
/// ```
/// # #[macro_use]
/// # extern crate render_tree;
/// use render_tree::prelude::*;
///
/// struct Header {
///     code: usize,
///     message: &'static str,
/// }
///
/// impl Render for Header {
///     fn render(self, document: Document) -> Document {
///         document.add(tree! {
///             {self.code} {": "} {self.message}
///         })
///     }
/// }
///
/// # fn main() -> ::std::io::Result<()> {
/// let code = 1;
/// let message = "Something went wrong";
///
/// let document = tree! {
///     <Header code={code} message={message}>
/// };
///
/// assert_eq!(document.to_string()?, "1: Something went wrong");
/// #
/// # Ok(())
/// # }
/// ```
///
/// # Block Components
///
/// You can also build components that take a block that runs exactly
/// once (an [`FnOnce`]).
///
/// ```
/// #[macro_use]
/// extern crate render_tree;
/// use render_tree::prelude::*;
///
/// struct Message {
///     code: usize,
///     message: &'static str,
///     trailing: &'static str,
/// }
///
/// impl BlockComponent for Message {
///     fn append(
///         self,
///         block: impl FnOnce(Document) -> Document,
///         mut document: Document,
///     ) -> Document {
///         document = document.add(tree! {
///             {self.code} {": "} {self.message} {" "}
///         });
///
///         document = block(document);
///
///         document = document.add(tree! {
///             {self.trailing}
///         });
///
///         document
///     }
/// }
///
/// # fn main() -> ::std::io::Result<()> {
/// let code = 1;
/// let message = "Something went wrong";
///
/// let document = tree! {
///     <Message code={code} message={message} trailing={" -- yikes!"} as {
///         {"!!! It's really quite bad !!!"}
///     }>
/// };
///
/// assert_eq!(document.to_string()?, "1: Something went wrong !!! It's really quite bad !!! -- yikes!");
/// #
/// # Ok(())
/// # }
/// ```
///
/// # Iterators
///
/// Finally, you can create components that take a block and call the block
/// multiple times (an iterator).
///
/// ```
/// # #[macro_use]
/// # extern crate render_tree;
/// use render_tree::prelude::*;
/// use std::io;
///
/// pub struct UpcaseAll<Iterator: IntoIterator<Item = String>> {
///     pub items: Iterator,
/// }
///
/// impl<Iterator: IntoIterator<Item = String>> IterBlockComponent for UpcaseAll<Iterator> {
///     type Item = String;
///
///     fn append(
///         self,
///         mut block: impl FnMut(String, Document) -> Document,
///         mut document: Document,
///     ) -> Document {
///         for item in self.items {
///             document = block(item.to_uppercase(), document);
///         }
///
///         document
///     }
/// }
///
/// # fn main() -> io::Result<()> {
/// let list = vec![format!("Hello"), format!("World")];
///
/// let document = tree! {
///     <UpcaseAll items={list} as |item| {
///         {"upcase:"} {item}
///     }>
/// };
///
/// assert_eq!(document.to_string()?, "upcase:HELLOupcase:WORLD");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! tree {
    // We're effectively handling patterns of matched delimiters that aren't intrinsically
    // supported by Rust here.
    //
    // If the first character we're processing is a `<`, that means we're looking at a
    // component of some kind. This macro matches a list of individual tokens, and
    // delegates the stuff between matching `< ... >`.
    {
        trace = [ $($trace:tt)* ]
        rest = [[ < $name:ident $($rest:tt)* ]]
    } => {
        tagged_element! {
            trace = [ $($trace)* { tagged_element } ]
            name = $name
            args=[]
            rest=[[ $($rest)* ]]
        }
    };

    // Anything other than an identifier immediately following a `<` is an error.
    {
        trace = [ $($trace:tt)* ]
        rest = [[ < $token:tt $($rest:tt)* ]]
    } => {{
        unexpected_token!(concat!("Didn't expect ", stringify!($token), "after `<`. A component must begin with an identifier"), trace = $trace, tokens = $token)
    }};

    // An empty stream after `<` is an unexpected EOF
    {
        trace = $trace:tt
        rest = [[ < ]]
    } => {{
        unexpected_eof!("Unexpected end of block immediately following `<`", trace = $trace)
    }};

    // If we didn't see a component, we're matching a single token, which must
    // correspond to an expression that produces an impl Render.
    {
        trace = [ $($trace:tt)* ]
        rest = [[ $token:tt $($rest:tt)* ]]
    } => {{
        let left = $crate::Render::into_fragment($token);

        let right = tree! {
            trace = [ $($trace)* { next token } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(left, right)
    }};

    // If there's no tokens left, produce Empty, which can be concatenated to
    // the end of any other produced `Render`s.
    {
        trace = $trace:tt
        rest = [[  ]]
    } => {
        $crate::Empty
    };

    // Anything else is an unexpected token, but since a previous rule matches
    // any `$token:tt`, it's not obvious what could match here.
    {
        trace = $trace:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unexpected_token!("Unexpected token in tree!", trace = $trace, tokens = $($rest)*)
    };

    // The entry point of the entire macro from the user.
    ($($rest:tt)*) => {
        tree! {
            trace = [ { tree } ]
            rest = [[ $($rest)* ]]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_token {
    ($message:expr,trace = $trace:tt,tokens = $token:tt $($tokens:tt)*) => {{
        force_mismatch!($token);
        macro_trace!($message, $trace);
    }};

    ($message:expr,trace = $trace:tt,tokens =) => {{
        unexpected_eof!($message, $trace);
    }};

    ($($rest:tt)*) => {{
        compile_error!("Invalid call to unexpected_token");
    }};
}

#[doc(hidden)]
#[allow(unused_macros)]
#[macro_export]
macro_rules! macro_trace {
    ($message:expr, [ $({ $($trace:tt)* })* ]) => {{
        compile_error!(concat!(
            $message,
            "\nMacro trace: ",

            $(
                $(
                    stringify!($trace),
                    " ",
                )*
                "-> ",
            )*
        ))
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! force_mismatch {
    () => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! unimplemented_branch {
    ($message:expr, trace = $trace:tt,tokens = $($tokens:tt)*) => {{
        unexpected_token!(concat!("Unimplemented branch: ", $message), trace = $trace, tokens = $($tokens)*);
    }};

    ($($rest:tt)*) => {{
        compile_error("Invalid call to unimplemented_branch");
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_eof {
     { $message:expr, trace = [ $($trace:tt)* ] } => {
        compile_error!(concat!("Unexpected end of block: ", $message, "\nMacro trace: ", stringify!($($trace)*)))
    };

    ($($rest:tt)*) => {{
        compile_error("Invalid call to unexpected_eof");
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! concat_trees {
    ($left:tt,()) => {
        $left
    };

    ((), $right:tt) => {
        $right
    };

    ($left:tt, $right:tt) => {{
        let mut document = $crate::Document::empty();
        document = $crate::Render::render($left, document);
        document = $crate::Render::render($right, document);

        document
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! tagged_element {
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ { args = $value:tt } ]
        rest = [[ > $($rest:tt)*]]
    } => {{
        let left = $crate::Component($name, $value);

        let rest =  tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(left, rest)
    }};

    // The `key={value}` syntax is only compatible with block-based components,
    // so if we see a `>` at this point, it's an error.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ > $($rest:tt)*]]
    } => {{
        let component = $name {
            $(
                $key: $value,
            )*
        };

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(component, rest)
    }};

    // Triage the next token into a "double token" because it may indicate an
    // error. If it turns out to be an error, we wil have the token as a
    // variable that we can get span reporting for.
    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ $maybe_block:tt $($rest:tt)* ]]
    } => {{
        tagged_element! {
            trace = $trace
            name = $name
            args = $args
            double = [[ @double << $maybe_block $maybe_block >> $($rest)*  ]]
        }
    }};

    // If we see a block, it's a mistake. Either the user forgot the name of
    // the key for an argument or they forgot the `as` prefix to a block.
    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        double = [[ @double << $maybe_block:tt { $(maybe_block2:tt)* } >> $($rest:tt)*  ]]
    } => {{
        unexpected_token!(
            concat!(
                "Pass a block to ",
                stringify!($name),
                " with the `as` keyword: `as` { ... } or pass args with args={ ... }"
            ),
            trace = $trace,
            tokens = $name
        );
    }};

    // If we see an `as`, we're looking at a block component.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = $args:tt
        double = [[ @double << $as:tt as >> $($rest:tt)*  ]]
    } => {{
        block_component!(
            trace = [ $($trace)* { block_component } ]
            name = $name
            args = $args
            rest = [[ $($rest)* ]]
        )
    }};

    // // Otherwise, if we see `args=`, it's the special singleton `args` case.
    // {
    //     trace = [ $($trace:tt)* ]
    //     name = $name:tt
    //     args = $args:tt
    //     double = [[ @double << args args >> = $($rest:tt)*  ]]
    // } => {{
    //     component_with_args! {
    //         trace = [ $($trace)* { component_with_args } ]
    //         name = $name
    //         rest = [[ $($rest)* ]]
    //     }
    // }};

    // Otherwise, if we see an `ident=`, we're looking at a key of an
    // argument. TODO: Combine this case with the previous one.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = $args:tt
        double = [[ @double << $key:ident $key2:ident >> = $($rest:tt)*  ]]
    } => {{
        tagged_element_value! {
            trace = [ $($trace)* { tagged_element_values } ]
            name = $name
            args = $args
            key = $key
            rest = [[ $($rest)* ]]
        }
    }};

    // Anything else is an error.
    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        double = [[ @double << $token:tt $double:tt >> $($rest:tt)* ]]
    } => {{
        unexpected_token!(concat!("Unexpected tokens after <", stringify!($name), ". Expected `key=value`, `as {` or `as |`"), trace = $trace, tokens = $token);
    }};

    // No more tokens is an error
    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ ]]
    } => {{
        unexpected_eof!(
            concat!("Unexpected end of block after <", stringify!($name)),
            trace = $trace
        );
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! tagged_element_value {
    // We saw a `ident=` and are now looking for a value.
    {
        trace = $trace:tt
        name = $name:tt
        args = [ $($args:tt)* ]
        key = $key:ident
        rest = [[ $value:ident $($rest:tt)* ]]
    } => {
        unexpected_token!(
            concat!(
                "Unexpected value ",
                stringify!($value),
                ". The value must be enclosed in {...}. Did you mean `",
                stringify!($key),
                "={",
                stringify!($value),
                "}`?"
            ),
            trace = $trace,
            tokens = $value
        );
    };

    // We saw a `ident=` and found a block. Accumulate the key/value pair and
    // continue parsing the tag.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $($args:tt)* ]
        key = $key:ident
        rest = [[ $value:block $($rest:tt)* ]]
    } => {
        tagged_element! {
            trace = [ $($trace)* { tagged_element } ]
            name = $name
            args = [ $($args)* { $key = $value } ]
            rest = [[ $($rest)*]]
        }
    };

    // Anything else is an error.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $($args:tt)* ]
        key = $key:ident
        rest = [[ $value:tt $($rest:tt)* ]]
    } => {
        tagged_element! {
            trace = [ $($trace)* { tagged_element } ]
            name = $name
            args = [ $($args)* { $key = $value } ]
            rest = [[ $($rest)*]]
        }
    };
}

// We got to the end of the tag opening and now we found a block. Parse
// the contents of the block as a new tree, and then continue processing
// the tokens.
#[doc(hidden)]
#[macro_export]
macro_rules! block_component {
    // If there were no arguments, call the function with the inner block.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = []
        rest = [[ { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        let inner = tree! {
            trace = [ $($trace)* { inner tree } ]
            rest = [[ $($block)* ]]
        };

        let component = $name(inner);

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(component, rest)
    }};

    // Otherwise, if there were arguments and closure parameters, construct
    // the argument object with the component's name and supplied arguments.
    // Then, call the component function with the constructed object and a
    // closure that takes a component-supplied callback parameter.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ |$id:tt| { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        let component = $name {
            $(
                $key: $value
            ),*
        };

        let block = $name::with(
            component, |$id, doc: $crate::Document| -> $crate::Document {
                (tree! {
                    trace = [ $($trace)* { inner tree } ]
                    rest = [[ $($block)* ]]
                }).render(doc)
            }
        );

        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(block, rest)
    }};

    // Otherwise, if there were arguments, construct the argument object
    // with the component's name and supplied arguments, and call the
    // function with a closure that doesn't take a user-supplied parameter.
    {
        trace = [ $($trace:tt)* ]
        name = $name:tt
        args = [ $({ $key:ident = $value:tt })* ]
        rest = [[ { $($block:tt)* }> $($rest:tt)* ]]
    } => {{
        let data = $name {
            $(
                $key: $value,
            )*
        };

        let block = |document: Document| -> Document {
            (tree! {
                trace = [ $($trace)* { inner tree } ]
                rest = [[ $($block)* ]]
            }).render(document)
        };

        let component = $crate::BlockComponent::with(data, block);


        let rest = tree! {
            trace = [ $($trace)* { rest tree } ]
            rest = [[ $($rest)* ]]
        };

        concat_trees!(component, rest)
    }};

    {
        trace = $trace:tt
        name = $name:tt
        args = $args:tt
        rest = [[ $($rest:tt)* ]]
    } => {
        unexpected_token!("Expected a block or closure parameters after `as`", trace = $trace, tokens=$($rest)*)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_usage() -> ::std::io::Result<()> {
        let hello = "hello";
        let world = format!("world");
        let answer = 42;

        let document = tree! {
            {hello} {" "} {world} {". The answer is "} {answer}
        };

        assert_eq!(document.to_string()?, "hello world. The answer is 42");

        Ok(())
    }
}
