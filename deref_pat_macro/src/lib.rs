mod ident;
mod transform;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::Expr;
use transform::transform;

/// Enables use of deref patterns in `if let` expressions.
///
/// For more information see the `deref_pat` crate.
///
/// # Usage
/// ```ignore
/// deref_pat! {
///     if let Foo { #[deref] string: bound @ "foo" } = &foo {
///         assert_eq!(bound, "foo");
///     } else {
///         panic!("did not match");
///     }
/// }
/// ```

#[proc_macro]
pub fn deref_pat(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as Expr);
    let out = transform(parsed);
    quote! { #out }.into()
}
