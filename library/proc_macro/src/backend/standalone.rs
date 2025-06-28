use std::cell::RefCell;
use std::ops::{Bound, Range};
use std::ptr;

use super::{bridge, server};
use crate::backend::rpc::DecodeMut;
use crate::backend::support::{Buffer, Closure};
use crate::bridge::client::Bridge;
use crate::bridge::client::state::BRIDGE_STATE;
use crate::bridge::{Diagnostic, ExpnGlobals, Literal, TokenTree};

pub(crate) fn set() {
    // let state = RefCell::new(Foo {});

    // pub(super) fn set<'bridge, R>(state: &RefCell<Bridge<'bridge>>, f: impl FnOnce() -> R) -> R {
    struct RestoreOnDrop(*const ());
    impl Drop for RestoreOnDrop {
        fn drop(&mut self) {
            BRIDGE_STATE.set(self.0);
        }
    }

    println!("set called");

    let mut bind = dispatch;
    let new_state = RefCell::new(Bridge {
        cached_buffer: Buffer::new(),
        dispatch: Closure::from(&mut bind),
        globals: ExpnGlobals { def_site: DEF_SITE, call_site: CALL_SITE, mixed_site: MIXED_SITE },
    });
    let new_state = Box::new(new_state);

    let inner: *const RefCell<Bridge<'_, _>> = ptr::from_ref(&new_state);

    println!("inner: {inner:p}");

    let outer = BRIDGE_STATE.replace(inner.cast::<()>());

    Box::leak(new_state);

    // let _restore = RestoreOnDrop(outer);

    // f()
}

fn dispatch(buf: Buffer) -> Buffer {
    let mut reader = &buf[..];
    match bridge::api_tags::Method::decode(&mut reader, &mut ()) {
        bridge::api_tags::Method::FreeFunctions(free_functions) => todo!(),
        bridge::api_tags::Method::TokenStream(token_stream) => todo!(),
        bridge::api_tags::Method::Span(span) => todo!(),
        bridge::api_tags::Method::Symbol(symbol) => todo!(),
    }

    println!("{:#x?}", &*buf);
    buf
}

thread_local! {
    static SESSION_GLOBALS: SessionGlobals = const {
        SessionGlobals { interner: RefCell::new(Vec::new()) }
    };
}

struct SessionGlobals {
    interner: RefCell<Vec<Box<str>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Span(u32);
const DEF_SITE: Span = Span(1);
const CALL_SITE: Span = Span(2);
const MIXED_SITE: Span = Span(3);

struct Standalone {}

struct Ff;

#[derive(Clone)]
struct Ts {}

#[derive(Clone, Copy, PartialEq)]
struct Symbol(u32);

impl Symbol {
    fn intern(s: &str) -> Self {
        SESSION_GLOBALS.with(|sess| {
            let mut interner = sess.interner.borrow_mut();
            if let Some(idx) = interner.iter().position(|sym| sym.as_ref() == s) {
                return Self(idx as u32);
            }
            interner.push(Box::from(s));
            let idx: u32 = interner.len().try_into().expect("exceeded u32::MAX symbols");
            Self(idx)
        })
    }

    fn with_str<T>(self, f: impl FnOnce(&str) -> T) -> T {
        SESSION_GLOBALS.with(|sess| {
            let mut interner = sess.interner.borrow();
            f(&interner[self.0 as usize])
        })
    }
}

impl server::Types for Standalone {
    type FreeFunctions = Ff;
    type TokenStream = Ts;
    type Span = Span;
    type Symbol = Symbol;
}

impl server::Server for Standalone {
    fn globals(&mut self) -> ExpnGlobals<Self::Span> {
        ExpnGlobals { def_site: DEF_SITE, call_site: CALL_SITE, mixed_site: MIXED_SITE }
    }

    fn intern_symbol(string: &str) -> Self::Symbol {
        Symbol::intern(string)
    }

    fn with_symbol_string(symbol: &Self::Symbol, f: impl FnOnce(&str)) {
        symbol.with_str(f)
    }
}

impl server::FreeFunctions for Standalone {
    fn injected_env_var(&mut self, var: &str) -> Option<String> {
        todo!()
    }

    fn track_env_var(&mut self, var: &str, value: Option<&str>) {
        todo!()
    }

    fn track_path(&mut self, path: &str) {
        todo!()
    }

    fn literal_from_str(&mut self, s: &str) -> Result<Literal<Self::Span, Self::Symbol>, ()> {
        todo!()
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic<Self::Span>) {
        todo!()
    }
}

impl server::TokenStream for Standalone {
    fn is_empty(&mut self, self_: &Self::TokenStream) -> bool {
        todo!()
    }

    fn expand_expr(&mut self, self_: &Self::TokenStream) -> Result<Self::TokenStream, ()> {
        todo!()
    }

    fn from_str(&mut self, src: &str) -> Self::TokenStream {
        todo!()
    }

    fn to_string(&mut self, self_: &Self::TokenStream) -> String {
        todo!()
    }

    fn from_token_tree(
        &mut self,
        tree: TokenTree<Self::TokenStream, Self::Span, Self::Symbol>,
    ) -> Self::TokenStream {
        todo!()
    }

    fn concat_trees(
        &mut self,
        base: Option<Self::TokenStream>,
        trees: Vec<TokenTree<Self::TokenStream, Self::Span, Self::Symbol>>,
    ) -> Self::TokenStream {
        todo!()
    }

    fn concat_streams(
        &mut self,
        base: Option<Self::TokenStream>,
        streams: Vec<Self::TokenStream>,
    ) -> Self::TokenStream {
        todo!()
    }

    fn into_trees(
        &mut self,
        self_: Self::TokenStream,
    ) -> Vec<TokenTree<Self::TokenStream, Self::Span, Self::Symbol>> {
        todo!()
    }
}

impl server::Span for Standalone {
    fn debug(&mut self, self_: Self::Span) -> String {
        todo!()
    }

    fn parent(&mut self, self_: Self::Span) -> Option<Self::Span> {
        todo!()
    }

    fn source(&mut self, self_: Self::Span) -> Self::Span {
        todo!()
    }

    fn byte_range(&mut self, self_: Self::Span) -> Range<usize> {
        todo!()
    }

    fn start(&mut self, self_: Self::Span) -> Self::Span {
        todo!()
    }

    fn end(&mut self, self_: Self::Span) -> Self::Span {
        todo!()
    }

    fn line(&mut self, self_: Self::Span) -> usize {
        todo!()
    }

    fn column(&mut self, self_: Self::Span) -> usize {
        todo!()
    }

    fn file(&mut self, self_: Self::Span) -> String {
        todo!()
    }

    fn local_file(&mut self, self_: Self::Span) -> Option<String> {
        todo!()
    }

    fn join(&mut self, self_: Self::Span, other: Self::Span) -> Option<Self::Span> {
        todo!()
    }

    fn subspan(
        &mut self,
        self_: Self::Span,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> Option<Self::Span> {
        todo!()
    }

    fn resolved_at(&mut self, self_: Self::Span, at: Self::Span) -> Self::Span {
        todo!()
    }

    fn source_text(&mut self, self_: Self::Span) -> Option<String> {
        todo!()
    }

    fn save_span(&mut self, self_: Self::Span) -> usize {
        todo!()
    }

    fn recover_proc_macro_span(&mut self, id: usize) -> Self::Span {
        todo!()
    }
}

impl server::Symbol for Standalone {
    fn normalize_and_validate_ident(&mut self, string: &str) -> Result<Self::Symbol, ()> {
        todo!()
    }
}

// struct Dispatcher<S: Types> {
//     handle_store: HandleStore<S>,
//     server: S,
// }

// macro_rules! define_dispatcher_impl {
//     ($($name:ident {
//         $(fn $method:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret_ty:ty)?;)*
//     }),* $(,)?) => {
//         // FIXME(eddyb) `pub` only for `ExecutionStrategy` below.
//         pub trait DispatcherTrait {
//             // HACK(eddyb) these are here to allow `Self::$name` to work below.
//             $(type $name;)*

//             fn dispatch(&mut self, buf: Buffer) -> Buffer;
//         }

//         impl<S: Server> DispatcherTrait for Dispatcher<MarkedTypes<S>> {
//             $(type $name = <MarkedTypes<S> as Types>::$name;)*

//             fn dispatch(&mut self, mut buf: Buffer) -> Buffer {
//                 let Dispatcher { handle_store, server } = self;

//                 let mut reader = &buf[..];
//                 match bridge::api_tags::Method::decode(&mut reader, &mut ()) {
//                     $(bridge::api_tags::Method::$name(m) => match m {
//                         $(bridge::api_tags::$name::$method => {
//                             let mut call_method = || {
//                                 bridge::reverse_decode!(reader, handle_store; $($arg: $arg_ty),*);
//                                 $name::$method(server, $($arg),*)
//                             };
//                             // HACK(eddyb) don't use `panic::catch_unwind` in a panic.
//                             // If client and server happen to use the same `std`,
//                             // `catch_unwind` asserts that the panic counter was 0,
//                             // even when the closure passed to it didn't panic.
//                             let r = if thread::panicking() {
//                                 Ok(call_method())
//                             } else {
//                                 panic::catch_unwind(panic::AssertUnwindSafe(call_method))
//                                     .map_err(rpc::PanicMessage::from)
//                             };

//                             buf.clear();
//                             r.encode(&mut buf, handle_store);
//                         })*
//                     }),*
//                 }
//                 buf
//             }
//         }
//     }
// }

// bridge::with_api!(Self, self_, define_dispatcher_impl);
