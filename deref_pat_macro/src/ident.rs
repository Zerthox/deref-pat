use proc_macro2::Span;
use syn::Ident;

pub const PREFIX: &str = concat!("_", env!("CARGO_PKG_NAME"));

#[derive(Debug, Default)]
pub struct IdentGen {
    count: u64,
}

impl IdentGen {
    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn next(&mut self) -> Ident {
        let ident = Ident::new(&format!("{}{}", PREFIX, self.count), Span::mixed_site());
        self.count += 1;
        ident
    }
}
