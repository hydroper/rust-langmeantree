# langmeantree

The `langmeantree` crate provides an intuitive way to describe meanings of a language using dynamic dispatches and hierarchy definitions using an arena.

`langmeantree` may be used for language compiler codebases for representing meanings, also referred to as *symbol* references.

*Note: this crate is not yet implemented.*

## To do

Reference links:

- https://github.com/hydroper/rust-class/blob/master/crates/oop_inheritance_proc/src/lib.rs
- https://github.com/hydroper/as3parser/wiki/Symbol-solvers
- https://docs.rs/syn/latest/syn/
- https://docs.rs/proc-macro2/latest/proc_macro2/

Steps after parsing:

* [ ] Define `MeaningSymbol` in a semantic model using an arena and a factory
  * [ ] `MeaningSlot`
    * [ ] `name()`
    * [ ] `inherits()`
    * [ ] `set_inherits()`
    * [ ] `submeanings()`
    * [ ] `methods()`
  * [ ] `FieldSlot`
    * [ ] `name()`
    * [ ] `field_type()` annotation
    * [ ] `field_initializer()` expression
    * [ ] `is_ref()`
  * [ ] `MethodSlot`
    * [ ] `name()`
    * [ ] `defined_in()`
    * [ ] `attributes()`
    * [ ] `override_logic_mapping(): SharedMap<MeaningSlot, OverrideLogicMapping>`
  * [ ] `OverrideLogicMapping` structure
    * [ ] `override_code()`
    * [ ] `set_override_code()`
    * [ ] `override_logic_mapping()`
* [ ] Output `type ArenaName = Arena<__data__::Meaning>;`
* [ ] 1. Traverse all meanings in a first pass
  * [ ] Create a `MeaningSlot`, setting the inherited type properly.
  * [ ] If the inherited type failed to resolve, ignore that meaning (assuming the error was reported); otherwise
    * [ ] Contribute the meaning to the inherited type's list of submeanings.
* [ ] 2. Define the data module `__data__`
* [ ] 3. Traverse each *meaning*
  * [ ] 3.1 Define the data structure `__data__::MeaningName`
  * [ ] 3.2 Traverse each field
    * [ ] 3.2.1 Create a `FieldSlot`.
    * [ ] 3.2.2 Contribute that slot to the meaning.
    * [ ] 3.2.3 Store the default initializer expression in the slot.
    * [ ] 3.2.4 Store the type annotation in the slot.
    * [ ] 3.2.5 Store about whether the slot is a `ref` field or not.
    * [ ] 3.2.6 Define a getter (`x()`)
      * [ ] 3.2.6.1 For non `ref`
      * [ ] 3.2.6.2 For `ref`
      * [ ] Get value by reading the correct base (it is either `self.0` or `self.0` followed by multiple `.0` depending on the number of inherited meanings, followed by `upgrade().unwrap()` and surrounded by a match)
    * [ ] 3.2.7 Define a mutable getter (`x_mut()`)
      * [ ] 3.2.7.1 For `ref` (returns `::std::cell::RefMut<T>`)
      * [ ] Get value by reading the correct base similiarly to immutable getters
    * [ ] 3.2.8 Define a setter (`set_x()`)
      * [ ] 3.2.8.1 For non `ref`
      * [ ] 3.2.8.2 For `ref`
      * [ ] Set value by reading the correct base similiarly to getters
  * [ ] 3.3 Define the structure `MeaningName`, as in `#[derive(Clone)] struct MeaningName(Weak<__data__::TopLevelMeaning>);`, or as in `#[derive(Clone, PartialEq, Hash)] struct MeaningName(InheritedMeaning)` if there is an inherited meaning.
    * [ ] Implement `PartialEq`
    * [ ] Implement `Hash`
    * [ ] If the meaning inherits another meaning
      * [ ] Implement `Deref<Target = InheritedMeaning>`
  * [ ] 3.4 Define the constructor
    * [ ] 3.4.1 Define the constructor *initializer* code as an instance `__lmt__ctor()` method
    * [ ] 3.4.2 Prepend an `arena: &MeaningArena` parameter to the constructor's input (not to the `__lmt__ctor()` method).
    * [ ] 3.4.3 At the constructor output code, let `meaning` be a complex `MeaningName(TopLevelMeaning(arena.allocate(__data__::TopLevelMeaning { ... })))` (notice the meaning layers) allocation initializing all meaning variants's fields with their default values.
    * [ ] 3.4.4 If the meaning inherits another meaning
      * [ ] 3.4.4.1 At the constructor output code, invoke `InheritedMeaning::__lmt__ctor(meaning, ...arguments)`, passing all `super(...)` arguments.
    * [ ] 3.4.5 Contribute all constructor initializer code to the `__lmt_ctor()` method.
    * [ ] 3.4.6 Output a `meaning` return
    * [ ] 3.4.7 Output the constructor as a static `new` method.
  * [ ] 3.5 Traverse each method
    * [ ] Create a `MethodSlot` with the appropriate settings.
    * [ ] Contribute the method slot to the meaning.
    * [ ] For each `super.f(...)` call within the method's block
      * [ ] Lookup for a `f` method in the inherited meanings in descending order
      * [ ] If nothing found, report an error at that `super.f(...)` call; otherwise
        * [ ] Replace `super.f(...)` by `InheritedMeaning::f(self, ...)`
    * [ ] Parse the modified method's block as a statement sequence
    * [ ] If the method is marked as `override`
      * [ ] Lookup for a method with the same name in the inherited meanings in descending order
        * [ ] If nothing found, report an error at the method's identifier; otherwise
          * [ ] ...
    * [ ] Contribute the method to the output
  * [ ] 3.6 Contribute a `to::<T: TryInto<MeaningName>>()` method that uses `TryInto`
  * [ ] 3.7 Contribute an `is::<T>` method that uses `to::<T>().is_some()`
  * [ ] 3.8 Contribute a `From<MeaningName> for InheritedMeaning` implementation (covariant conversion)
  * [ ] 3.9 Contribute a `TryFrom<InheritedMeaning> for MeaningName` implementation (contravariant conversion)

*To do*: finish describing the overriding steps.

## Definition order

Definition order is sensitive. Define submeanings after their inherited meanings while using the `struct` keyword.

If you define in `struct`s in any order, you may get a compile-time error; luckily, it is easy to identify these cases as structures that were failed to be processed are ignored.

## Example

```rust
use langmeantree::langmeantree;

langmeantree! {
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
        // fn set_x(&self) -> f64
        let x: f64 = 0.0;

        // Use "ref" for RefCell. RefCell is used for data types
        // such as String, Vec, HashMap, Rc, and so on.
        //
        // fn y(&self) -> Ref<String>
        // fn y_mut(&self) -> RefMut<String>
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

It is recommended for fields to always start with a underscore `_`, and consequently using accesses such as `__x()`, `__x_mut()`, or `set__x(v)`.

## Submeanings

* `meaning.is::<T>()` tests whether `meaning` is a `T` submeaning.
* `meaning.to::<T>()` converts to the `T` meaning, returning `Ok(m)` or `Err`. It may be a contravariant conversion.
* `meaning.into()` is a covariant conversion.

## Super expression

The `super.f()` expression is supported by preprocessing the token sequence of a method and transforming it into another Rust code; therefore, it may be used anywhere within an instance method.

`super.f()` does a lookup in the method lists in the descending meanings.

## Documentation

Use the `#[inheritdoc]` attribute to inherit the RustDoc comment of an overriden method.