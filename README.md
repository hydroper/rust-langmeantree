# SModel

SModel (Semantic Modeling) for Rust provides an intuitive way to describe semantic symbols of a language using dynamic dispatches and hierarchy definitions using an arena that allows for circular references.

*Note: this crate is not yet implemented.*

## To do

Reference links:

<!--
- https://github.com/hydroper/rust-class/blob/master/crates/oop_inheritance_proc/src/lib.rs
-->

- https://docs.rs/syn/latest/syn/
- https://docs.rs/proc-macro2/latest/proc_macro2/
- https://docs.rs/quote/latest/quote/

Steps after parsing:

* [x] Define `Symbol` in a semantic model using an arena and a factory
* [x] 1. Output `type ArenaName = ::smodel::Arena<#DATA::Meaning>;`
* [x] 2. Traverse all meanings in a first pass
* [ ] 3. Traverse each meaning
  * [x] 3.1 Write out the base data accessor
  * [x] 3.2 Traverse each field
  * [x] 3.3 Contribute a `#DATA_VARIANT_FIELD` field to `#DATA::M` holding the enumeration of submeanings.
  * [x] 3.4 Contribute a `#[non_exhaustive]` enumeration of submeanings whose name is `submeaning_enum = DATA_VARIANT_PREFIX.to_owned() + meaning_name` at the `#DATA` module.
  * [x] 3.5. Define the data structure `#DATA::M` at the `#DATA` module output, containing all field output.
  * [x] 3.6 Define the structure `M`
  * [x] 3.7 Define the constructor
  * [x] 3.8 Traverse each method (see below section 3.8)
  * [ ] 3.9 Traverse each method
    * [ ] Skip if it is not mapped to an instance method slot.
    * [ ] Contribute the method `#method_name` with prepended dynamic dispatch logic, invoking `self.#nondispatch_name(#input_args)` at the end of the method body, to the output
  * [ ] 3.10 Contribute a `to::<T: TryInto<M>>()` method that uses `TryInto`
  * [ ] 3.11 Contribute an `is::<T>` method that uses `to::<T>().is_some()`
  * [ ] 3.12 Output the code of all methods to an `impl` block for the meaning data type.
* [ ] 4. Output the `mod #DATA { use super::*; ... }` module with its respective contents

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
