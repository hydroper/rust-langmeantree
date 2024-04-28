use crate::*;

pub struct ProcessingStep3_2();

impl ProcessingStep3_2 {
    pub fn exec(&self, host: &mut LmtHost, meaning: &Symbol, field: &Rc<MeaningField>, base_accessor: &str, asc_meaning_list: &[Symbol], field_output: &mut TokenStream) {
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
                pub #field_name: ::std::cell::RefCell<#field_type>,
            }.try_into().unwrap());
        } else {
            field_output.extend::<TokenStream>(quote! {
                pub #field_name: ::std::cell::Cell<#field_type>,
            }.try_into().unwrap());
        }

        // 4. Define accessors
        self.define_accessors(host, meaning, &slot, &field_name, &field_type, base_accessor, asc_meaning_list);
    }

    fn define_accessors(&self, _host: &mut LmtHost, meaning: &Symbol, slot: &Symbol, field_name: &str, field_type: &Type, base_accessor: &str, asc_meaning_list: &[Symbol]) {
        let setter_name = format!("set_{}", field_name);
        let fv = self.match_field(asc_meaning_list, 0, &format!("{base_accessor}.upgrade().unwrap()"), field_name);

        if slot.is_ref() {
            meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
                fn #field_name(&self) -> #field_type {
                    #fv.borrow().clone()
                }
                fn #setter_name(&self, v: #field_type) {
                    $fv.replace(v);
                }
            }.try_into().unwrap());
        } else {
            meaning.method_output().borrow_mut().extend::<TokenStream>(quote! {
                fn #field_name(&self) -> #field_type {
                    #fv.get()
                }

                fn #setter_name(&self, v: #field_type) {
                    #fv.set(v);
                }
            }.try_into().unwrap());
        }
    }

    /// Matches a field. `base` is assumed to be a `Rc<__data__::M>` value.
    fn match_field(&self, asc_meaning_list: &[Symbol], meaning_index: usize, base: &str, field_name: &str) -> String {
        let inherited = if asc_meaning_list.len() - meaning_index == 1 {
            None
        } else {
            Some(asc_meaning_list[meaning_index].clone())
        };
        let meaning = asc_meaning_list[meaning_index + if inherited.is_some() { 1 } else { 0 }].clone();

        let Some(inherited) = meaning.inherits() else {
            return format!("{}.{}", base, field_name);
        };
        format!("(if __data__::{}::{}(o) = &{base}.{DATA_VARIANT_FIELD} {{ {} }} else {{ panic!() }})",
            DATA_VARIANT_PREFIX.to_owned() + &inherited.name(),
            meaning.name(),
            self.match_field(asc_meaning_list, meaning_index + 1, "o", field_name))
    }
}