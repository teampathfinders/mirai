//! Provides custom macros for the Inferno server.

#![forbid(missing_docs)]

use proc_macro::TokenStream;

mod atomic_enum;
mod variant_count;

/// Generates a new type prefixed with `Atomic` that is the same as the affected
/// enum but provides atomic load and store operations.
#[proc_macro_attribute]
pub fn atomic_enum(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    atomic_enum::inner(item)
}

/// Creates a `variant_count` method that returns the amount of variants that the enum has.
/// This is a temporary hack until the `std::mem::variant_count` function is stabilized.
#[proc_macro_attribute]
pub fn variant_count(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    variant_count::inner(item)
}
