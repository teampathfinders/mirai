use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, DeriveInput};

/// Creates a `variant_count` method that returns the amount of variants that the enum has.
/// This is a temporary hack until the `std::mem::variant_count` function is stabilized.
pub fn inner(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    let Data::Enum(data) = &input.data else {
        return TokenStream::from(quote_spanned! {
            input.span() => compile_error!("variant_count can only be applied to enums")
        });
    };

    let count = data.variants.len();
    let ident = &input.ident;

    TokenStream::from(quote! {
        #input

        impl #ident {
            /// The amount of variants that this enum has.
            pub const fn variant_count() -> usize {
                #count
            }
        }
    })
}
