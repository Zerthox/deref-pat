# deref_pat
A [Rust](https://rust-lang.org) macro allowing to dereference struct fields in patterns.

You can use the macro to match on fields containing [`Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html), [`Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html), [`String`](https://doc.rust-lang.org/std/string/struct.String.html), [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) and everything else that implements [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html)/[`DerefMut`](https://doc.rust-lang.org/std/ops/trait.DerefMut.html).
In the special case of [`Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html) even matching on owned values is supported.

## Usage
```rs
deref_pat! {
    if let Foo { #[deref] string: bound @ "foo" } = &foo {
        assert_eq!(bound, "foo");
    } else {
        panic!("did not match");
    }
}
```

The generated code looks something like:
```rs
if let Some(bound) = {
    let mut result = None;
    if let Foo { string } = &foo {
        if let bound @ "foo" = PatDeref::pat_deref(string) {
            result = Some(bound);
        }
    }
    result
} {
    assert_eq!(bound, "foo");
} else {
    panic!("did not match");
}
```

## Notes
- The `deref_pat` crate must be in scope under this exact name. Supplying a custom name is not yet supported.
- Only supports `if let` expressions. `match` expression are not yet supported.
