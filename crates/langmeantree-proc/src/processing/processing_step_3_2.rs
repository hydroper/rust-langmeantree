use crate::*;

pub struct ProcessingStep3_2();

impl ProcessingStep3_2 {
    pub fn exec(&self, host: &mut LmtHost, meaning: &Symbol, field: &Rc<MeaningField>, base_accessor: &str, submeaning_enum: &str) {
        let mut field_output = TokenStream::new();

        // 1. Create a FieldSlot.
        let slot = host.factory.create_field_slot(field.is_ref, field.name.to_string(), field.type_annotation.clone(), field.default_value.clone());

        // 2. Contribute the field slot to the meaning slot.
        if meaning.fields().has(&slot.name()) {
            field.name.span().unwrap().error(format!("Redefining '{}'", slot.name())).emit();
            return;
        } else {
            meaning.fields().set(slot.name(), slot.clone());
        }

        // 3. Contribute a field to the __data__::M structure.
        let field_name = slot.name();
        let field_type = slot.field_type();
        if slot.is_ref() {
            field_output.extend::<TokenStream>(quote! {
                #field_name: ::std::cell::RefCell<#field_type>,
            }.try_into().unwrap());
        } else {
            field_output.extend::<TokenStream>(quote! {
                #field_name: ::std::cell::Cell<#field_type>,
            }.try_into().unwrap());
        }

        // 4. Define accessors
        self.define_accessors(host, meaning, &slot, &field_name, &field_type, base_accessor, submeaning_enum);
    }

    fn define_accessors(&self, host: &mut LmtHost, meaning: &Symbol, slot: &Symbol, field_name: &str, field_type: &Type, base_accessor: &str, submeaning_enum: &str) {
        let setter_name = format!("set_{}", field_name);

        if slot.is_ref() {
            let mut_getter_name = format!("{}_mut", field_name);

            meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
                pub fn #field_name(&self) -> #field_type {
                    //
                }
                pub fn #mut_getter_name(&self) -> #field_type {
                    //
                }
                pub fn #setter_name(&self, v: #field_type) {
                    //
                }
            }.try_into().unwrap());
        } else {
            meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
                pub fn #field_name(&self) -> #field_type {
                    //
                }

                pub fn #setter_name(&self, v: #field_type) {
                    //
                }
            }.try_into().unwrap());
        }
    }
}