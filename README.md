# SModel

SModel (Semantic Modeling) for Rust provides an intuitive way to describe semantic symbols of a language using dynamic dispatches and hierarchy definitions using an arena that allows for circular references.

*Note: this crate is almost implemented; just fixing bugs.*

## Definition order

Definition order is sensitive. Define submeanings after their inherited meanings while using the `struct` keyword.

If you define `struct`s in any order, you may get a compile-time error; luckily, it is easy to identify these cases as structures that were failed to be processed are ignored.

## Example

```rust
use smodel::smodel;

smodel! {
    type Arena = MeaningArena;

    struct Meaning {
        let x: f64 = 0.0;
        let ref y: String = "".into();

        pub fn Meaning() {
            super();
            println!("{}", this.m());
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

## Arena

The arena's name is defined as the right-hand side of the first `type Arena = ArenaName1;` directive.

## Fields

A field (a `let` declaration) has an optional `ref` modifier indicating whether to use `RefCell` or `Cell`. For all, types are either cloned or copied on read. Use `ref` for heap-allocated resources such as `String`.

Fields have a pair of a getter (`fieldname()`) and a setter (`set_fieldname(value)`).

For mutable hash maps or vectors, it is recommended to use a container that is cloned by reference and not by content.

Fields are always private to the meaning, therefore there are no attributes; the field definition always starts with the `let` keyword, without a RustDoc comment.

It is recommended for fields to always start with a underscore `_`, and consequently using accesses such as `_x()`, or `set__x(v)`.

Then, you would implement methods that override other methods in a base meaning, allowing for an *unified* data type that supports methods that operate on more than one variant.

## Constructor

The constructor is a method whose name matches the meaning's name. The `arena` parameter is implicitly prepended to the formal parameter list.

The constructor is translated to a static `new` method.

The constructor contains a local `this` variable whose data type is the instance of that meaning.

## Submeanings

* `meaning.is::<T>()` tests whether `meaning` is a `T` submeaning.
* `meaning.to::<T>()` converts to the `T` meaning, returning `Ok(m)` or `Err`. It may be a contravariant conversion.
* `meaning.into()` is a covariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

`super.f()` does a lookup in the method lists in the descending meanings.

## Inheriting documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.
