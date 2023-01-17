use proc_macro2::Span;
use syn::{Ident, Path, PathSegment};

/// Creates an [`Ident`] with mixed site [`Span`].
pub fn create_ident(name: impl AsRef<str>) -> Ident {
    Ident::new(name.as_ref(), Span::mixed_site())
}

/// Creates a [`Path`] with the given segments.
/// `global` prepends `::`.
pub fn create_path<I>(segments: impl IntoIterator<Item = I>, global: bool) -> Path
where
    I: AsRef<str>,
{
    Path {
        leading_colon: if global {
            Some(Default::default())
        } else {
            None
        },
        segments: segments
            .into_iter()
            .map(|item| PathSegment::from(create_ident(item)))
            .collect(),
    }
}
