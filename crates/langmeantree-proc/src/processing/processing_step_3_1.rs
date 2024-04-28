use crate::*;

pub struct ProcessingStep3_1();

impl ProcessingStep3_1 {
    pub fn exec(&self, host: &mut LmtHost, meaning: &Symbol, field: &Rc<MeaningField>) {
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

        // 4. Define a getter "x()"
        todo!();
    }
}