use crate::util::create_ident;
use syn::Ident;

#[derive(Debug, Default)]
pub struct IdentGen {
    count: u64,
}

impl IdentGen {
    pub const PREFIX: &str = "deref_pat_";

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn next(&mut self) -> Ident {
        let ident = Self::prefix(self.count.to_string());
        self.count += 1;
        ident
    }

    pub fn prefix(ident: impl AsRef<str>) -> Ident {
        create_ident(Self::PREFIX.to_string() + ident.as_ref())
    }
}
