mod support;

#[unstable(feature = "proc_macro_internals", issue = "27812")]
pub mod bridge;
#[unstable(feature = "proc_macro_internals", issue = "27812")]
pub mod client;
pub(crate) mod rpc;
#[unstable(feature = "proc_macro_internals", issue = "27812")]
pub mod server;

mod symbol;

mod standalone;

pub(crate) fn enable_standalone() {
    standalone::set();
}
