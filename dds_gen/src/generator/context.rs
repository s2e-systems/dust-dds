use std::{borrow::Cow, fmt::Write};

/// Generator context.
#[derive(Debug)]
pub struct Context<W: Write> {
    /// Writer.
    pub writer: W,
    /// List of modules to keep track hierarchy.
    pub modules: Vec<String>,
}

impl<W: Write> Context<W> {
    /// Module separator.
    pub const MODULE_SEP: &str = "::";

    /// Constructs a new `Context` from `writer`.
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            modules: Vec::default(),
        }
    }

    /// Returns the type name of `ident`, with all modules joined by [`Context::MODULE_SEP`].
    pub fn type_name<'a>(&'a self, ident: &'a str) -> Cow<'a, str> {
        if self.modules.is_empty() {
            return Cow::Borrowed(ident);
        }

        Cow::Owned(format!(
            "{}{}{ident}",
            self.modules.join(Self::MODULE_SEP),
            Self::MODULE_SEP
        ))
    }
}

impl<W: Write + Default> Default for Context<W> {
    #[inline]
    fn default() -> Self {
        Self::new(W::default())
    }
}
