use std::fmt;

pub(crate) struct CommaArray<I: Into<String> + Clone>(pub(crate) Vec<I>);

impl<I: Into<String> + Clone> fmt::Display for CommaArray<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;

        if self.0.len() > 0 {
            let last = self.0.len() - 1;

            for (i, item) in self.0.clone().into_iter().enumerate() {
                let item: String = item.into();
                write!(f, "{}", item)?;

                if i != last {
                    write!(f, ", ")?;
                }
            }
        }

        write!(f, "]")
    }
}
