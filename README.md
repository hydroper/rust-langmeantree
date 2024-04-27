# Meaning trees for Rust

The `langmeantree` crate provides an intuitive way to describe meanings of a language using dynamic dispatches and hierarchy definitions using an arena.

`langmeantree` may be used for language compiler codebases.

## Example

```rust
use langmeantree::{Arena, langmeantree};

langmeantree! {
    type Arena = MeaningArena;

    struct Meaning {
        // fn x() -> f64
        // fn set_x(x: f64) -> Self
        let x: f64 = 0.0;

        // Use the "ref" keyword for heap resources
        // such as String and Vec.
        //
        // fn x() -> String
        // fn set_x(x: String) -> Self
        let ref y: String = "".into();

        // The constructor; it is called as `Meaning::new(&arena, ...arguments)`.
        pub fn Meaning() {
        }

        pub fn m(&self) -> String {
            "".into()
        }

        pub fn m1(&self) {
            println!("base");
        }
    }

    struct FooMeaning: Meaning {
        pub fn FooMeaning() {
            super();
        }

        pub override fn m(&self) -> String {
            "Foo".into()
        }

        pub override fn m1(&self) {
            if true {
                super.m1();
            }
        }
    }
}

fn main() {
    let arena = MeaningArena::new();
    let m = FooMeaning::new(&arena);
    println!("{}", m.m());
}
```

## Submeanings

* `meaning.is::<T>()` tests whether `meaning` is a `T` submeaning.
* `meaning.to::<T>()` converts to the `T` meaning, returning `Ok(m)` or `Err`. It may be either a covariant or contravariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

## Documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.