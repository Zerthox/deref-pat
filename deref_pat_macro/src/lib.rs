mod ident;
mod transform;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::Expr;
use transform::transform;

/// Enables use of deref patterns in `if let` expressions.
#[proc_macro]
pub fn deref_pat(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as Expr);
    let out = transform(parsed);
    quote! { #out }.into()
}
