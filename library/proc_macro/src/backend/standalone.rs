use std::cell::RefCell;
use std::ptr;

use crate::backend::support::Buffer;
use crate::bridge::ExpnGlobals;
use crate::bridge::client::Bridge;
use crate::bridge::client::state::BRIDGE_STATE;

pub fn set() {
    // let state = RefCell::new(Foo {});

    // pub(super) fn set<'bridge, R>(state: &RefCell<Bridge<'bridge>>, f: impl FnOnce() -> R) -> R {
    struct RestoreOnDrop(*const ());
    impl Drop for RestoreOnDrop {
        fn drop(&mut self) {
            BRIDGE_STATE.set(self.0);
        }
    }

    let new_state = RefCell::new(Bridge {
        cached_buffer: Buffer::new(),
        dispatch: todo!(),
        globals: ExpnGlobals { def_site: DEF_SITE, call_site: CALL_SITE, mixed_site: MIXED_SITE },
    });
    let new_state = Box::new(new_state);

    let inner = ptr::from_ref(&new_state).cast();

    let outer = BRIDGE_STATE.replace(inner);

    Box::leak(new_state);

    let _restore = RestoreOnDrop(outer);

    // f()
}

struct Span(u32);
const DEF_SITE: Span = Span(1);
const CALL_SITE: Span = Span(2);
const MIXED_SITE: Span = Span(3);
