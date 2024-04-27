# Meaning trees for Rust

The `langmeantree` crate provides an intuitive way to describe meanings of a language using dynamic dispatches and hierarchy definitions using an arena.

`langmeantree` may be used for language compiler codebases.

## To do

Reference links:

- https://github.com/hydroper/rust-class/blob/master/crates/oop_inheritance_proc/src/lib.rs
- https://github.com/hydroper/as3parser/wiki/Symbol-solvers
- https://docs.rs/syn/latest/syn/
- https://docs.rs/proc-macro2/latest/proc_macro2/

## Example

```rust
use langmeantree::{Arena, langmeantree};

langmeantree! {
    type Arena = MeaningArena;

    struct Meaning {
        // Fields are cloned when read, by default;
        // thus, assuming the variable's data type implements
        // Clone.
        //
        // fn x(&self) -> f64
        let x: f64 = 0.0;

        // Use "mut" for Cell.
        //
        // fn y(&self) -> f64
        // fn set_y(&self, value: f64) -> Self
        let mut y: f64 = 0.0;

        // Use "mut ref" for RefCell.
        //
        // fn z(&self) -> String
        // fn set_z(&self, value: String) -> Self
        let mut ref z: String = "".into();

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
    let meaning = FooMeaning::new(&arena);
    println!("{}", meaning.m());
}
```

## Fields

Fields are always private to the meaning, therefore there are no attributes; the field definition always starts with the `let` keyword, without a RustDoc comment.

## Submeanings

* `meaning.is::<T>()` tests whether `meaning` is a `T` submeaning.
* `meaning.to::<T>()` converts to the `T` meaning, returning `Ok(m)` or `Err`. It may be either a covariant or contravariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

## Documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.