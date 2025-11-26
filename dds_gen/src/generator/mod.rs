pub mod rust;

pub use self::rust::RustGenerator;
use crate::parser::IdlPair;
use std::borrow::Cow;

/// Generator trait.
pub trait Generator {
    /// Module separator.
    const MODULE_SEP: &str = "::";

    /// Generate source code.
    fn generate(&mut self, pair: IdlPair);

    /// Returns the list of current modules.
    fn modules(&self) -> &[String];

    /// Returns the type name of `ident`, with all modules joined by [`Generator::MODULE_SEP`].
    fn type_name<'a>(&'a self, ident: &'a str) -> Cow<'a, str> {
        let modules = self.modules();

        if modules.is_empty() {
            return Cow::Borrowed(ident);
        }

        Cow::Owned(format!(
            "{}{}{ident}",
            modules.join(Self::MODULE_SEP),
            Self::MODULE_SEP
        ))
    }
}
