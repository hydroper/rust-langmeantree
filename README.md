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
* [x] 1. Output `type ArenaName = ::smodel::Arena<__data__::Meaning>;`
* [x] 2. Traverse all meanings in a first pass
* [ ] 3. Traverse each meaning
  * [x] 3.1 Write out the base data accessor
  * [x] 3.2 Traverse each field
  * [x] 3.3 Contribute a `#DATA_VARIANT_FIELD` field to `__data__::M` holding the enumeration of submeanings.
  * [x] 3.4 Contribute a `#[non_exhaustive]` enumeration of submeanings whose name is `submeaning_enum = DATA_VARIANT_PREFIX.to_owned() + meaning_name` at the `__data__` module.
  * [x] 3.5. Define the data structure `__data__::M` at the `__data__` module output, containing all field output.
  * [x] 3.6 Define the structure `M`
  * [ ] 3.7 Define the constructor (see below section 3.7)
  * [ ] 3.8 Traverse each method
    * [ ] Create a `MethodSlot` with the appropriate settings.
    * [ ] Contribute the method slot to the meaning.
    * [ ] Check if the method has a `#[inheritdoc]` attribute; if it has one
      * [ ] Remove it
      * [ ] Lookup method in one of the base meanings
      * [ ] Inherit documentation comments
    * [ ] For each `super.f(...)` call within the method's block
      * [ ] Lookup for a `f` method in the inherited meanings in descending order
      * [ ] If nothing found, report an error at that `super.f(...)` call; otherwise
        * [ ] Let `base` be `self.clone()` surrounded by each submeaning's structure in the correct order.
        * [ ] Replace `super.f(...)` by `InheritedM::__sm_nondispatch_f(#base, ...)`
    * [ ] Parse the modified method's block as a statement sequence
    * [ ] If the method is marked as `override`
      * [ ] Lookup for a method with the smodele name in the inherited meanings in descending order
        * [ ] If nothing found, report an error at the method's identifier; otherwise
          * [ ] Contribute "overriding" return call code to the respective override logic mapping according to meaning inheritance
    * [ ] Contribute the method `__sm_nondispatch_m` without dynamic dispatch to the output
    * [ ] Contribute the method `m` with dynamic dispatch, invoking `self.__sm_nondispatch_m()` at the end of the method body, to the output
  * [ ] 3.9 Contribute a `to::<T: TryInto<M>>()` method that uses `TryInto`
  * [ ] 3.10 Contribute an `is::<T>` method that uses `to::<T>().is_some()`
  * [ ] 3.11 Output the code of all methods to an `impl` block for the meaning data type.
* [ ] 4. Output the `mod __data__ { use super::*; ... }` module with its respective contents

#### 3.7 Constructor

* [ ] 1. Define the constructor *initializer* code as an instance `__sm__ctor()` method
* [ ] 2. Prepend an `arena: &MeaningArena` parameter to the constructor's input (not to the `__sm__ctor()` method).
* [ ] 3. At the constructor output code, let `meaning` be a complex `M2(M1(arena.allocate(__data__::M1 { ... })))` (notice the meaning layers) allocation initializing all meaning variants's fields with their default values.
* [ ] 4. If the meaning inherits another meaning
  * [ ] 4.1. At the constructor output code, invoke `InheritedM::__sm__ctor(meaning, ...arguments)`, passing all `super(...)` arguments.
* [ ] 5. Contribute all constructor initializer code to the `__sm_ctor()` method.
* [ ] 6. Output a `meaning` return
* [ ] 7. Output the constructor as a static `new` method.

## Definition order

Definition order is sensitive. Define submeanings after their inherited meanings while using the `struct` keyword.

If you define in `struct`s in any order, you may get a compile-time error; luckily, it is easy to identify these cases as structures that were failed to be processed are ignored.

## Example

```rust
use smodel::smodel;

smodel! {
    type Arena = MeaningArena;

    struct Meaning {
        // Fields use Cell and are copied when read, by default;
        // therefore assuming the variable's data type implements
        // Copy.
        //
        // Use this form for primitive types such as f64, u32,
        // usize, bool, and so on.
        //
        // fn x(&self) -> f64
        // fn set_x(&self, value: f64)
        let x: f64 = 0.0;

        // Use "ref" for RefCell. RefCell is used for data types
        // such as String, Vec, HashMap, Rc, and so on.
        //
        // Note that this will assume that the data type implements Clone,
        // as the methods clone data on read.
        //
        // For semantic cases that need mutability, use a
        // shared container that is cloned by reference.
        //
        // fn y(&self) -> String
        // fn set_y(&self, value: String) -> Self
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
    let meaning = FooMeaning::new(&arena);
    println!("{}", meaning.m());
}
```

## Fields

Fields are always private to the meaning, therefore there are no attributes; the field definition always starts with the `let` keyword, without a RustDoc comment.

It is recommended for fields to always start with a underscore `_`, and consequently using accesses such as `__x()`, or `set__x(v)`.

## Submeanings

* `meaning.is::<T>()` tests whether `meaning` is a `T` submeaning.
* `meaning.to::<T>()` converts to the `T` meaning, returning `Ok(m)` or `Err`. It may be a contravariant conversion.
* `meaning.into()` is a covariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

`super.f()` does a lookup in the method lists in the descending meanings.

## Documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.
