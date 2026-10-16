use crate::presenter::Ui;
use adw::prelude::*;
use std::cell::RefCell;
thread_local! { static OWNER: RefCell<Option<Ui>> = const { RefCell::new(None) }; }
#[derive(Debug, PartialEq)]
enum Decision {
    Create,
    Present,
    WaitForDrain,
}
fn decide(owned: bool, closing: bool) -> Decision {
    if !owned {
        Decision::Create
    } else if closing {
        Decision::WaitForDrain
    } else {
        Decision::Present
    }
}
pub(crate) fn activate(app: &adw::Application) {
    OWNER.with(|slot| {
        let existing = slot.borrow().clone();
        match decide(
            existing.is_some(),
            existing.as_ref().is_some_and(|ui| ui.closing.get()),
        ) {
            Decision::Create => *slot.borrow_mut() = Some(crate::composition::build(app)),
            Decision::Present => existing.unwrap().window.present(),
            Decision::WaitForDrain => crate::diagnostics::record(format_args!(
                "event=activation_deferred_for_provider_drain"
            )),
        }
    });
}
pub(crate) fn retired() {
    OWNER.with(|slot| {
        slot.borrow_mut().take();
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activation_never_creates_an_owner_while_old_work_drains() {
        assert_eq!(decide(false, false), Decision::Create);
        assert_eq!(decide(true, false), Decision::Present);
        assert_eq!(decide(true, true), Decision::WaitForDrain);
        assert_eq!(decide(false, true), Decision::Create);
    }
}
