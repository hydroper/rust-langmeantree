use crate::*;

pub struct ProcessingStep2();

impl ProcessingStep2 {
    pub fn exec(&self, host: &mut SModelHost, m: &Rc<Meaning>) {
        // 1. Create a MeaningSlot.
        let slot = host.factory.create_meaning_slot(m.name.to_string());

        // 1.2. Resolve the inherited meaning.
        // 1.3. If the inherited meaning failed to resolve, ignore that meaning
        // (assuming the error was reported); otherwise
        // 1.3.1. Contribute the meaning to the inherited type's list of submeanings.
        if let Some(inherits) = &m.inherits {
            if let Some(inherited_meaning) = host.meaning_slots.get(&inherits.to_string()) {
                slot.set_inherits(Some(inherited_meaning));
                inherited_meaning.submeanings().push(slot.clone());
            } else {
                inherits.span().unwrap().error(format!("Data type '{}' not found.", inherits.to_string())).emit();
                return;
            }
        }

        // 1.4. Contribute meaning slot to the set of known meaning slots.
        if host.meaning_slots.contains_key(&slot.name()) {
            m.name.span().unwrap().error(format!("Redefining '{}'", slot.name())).emit();
        } else {
            host.meaning_slots.insert(slot.name(), slot.clone());
        }

        // 1.5. Map the meaning node to the meaning slot.
        host.semantics.set(m, Some(slot));
    }
}