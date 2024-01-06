use std::convert::Infallible;

use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse::{Parse, ParseStream, Result}, Visibility, Ident, Token, Block, Attribute, Meta, DeriveInput, Expr, spanned::Spanned, Data, Fields};

// struct AtomicEnumInput {
//     repr: String,
//     enum_size: usize,
//     visibility: Visibility,
//     name: Ident,
//     items: Block
// }

// impl Parse for AtomicEnumInput {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let attributes: Vec<Attribute> = input.call(Attribute::parse_outer)?;
//         let mut repr = String::from("u32");

//         for attr in attributes {
//             if attr.path().is_ident("repr") {
//                 attr.parse_nested_meta(|meta| {
//                     if meta.path.is_ident("i8") { repr = String::from("i8"); }
//                     if meta.path.is_ident("i16") { repr = String::from("i16"); }
//                     if meta.path.is_ident("i32") { repr = String::from("i32"); }
//                     if meta.path.is_ident("i64") { repr = String::from("i64"); }
//                     if meta.path.is_ident("u8") { repr = String::from("u8"); }
//                     if meta.path.is_ident("u16") { repr = String::from("u16"); }
//                     if meta.path.is_ident("u32") { repr = String::from("u32"); }
//                     if meta.path.is_ident("u64") { repr = String::from("u64"); }
//                     if meta.path.is_ident("isize") { repr = String::from("isize"); }
//                     if meta.path.is_ident("usize") { repr = String::from("usize"); }

//                     Ok(())
//                 })?;

//                 break
//             }
//         }

//         let visibility: Visibility = input.parse()?;
//         input.parse::<Token![enum]>()?;

//         let name: Ident = input.parse()?;
//         let items: Block = input.parse()?;

//         Ok(Self {
//             repr, visibility, name, items
//         })
//     }
// }

#[proc_macro_attribute]
pub fn atomic_enum(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    // let AtomicEnumInput {
    //     repr, enum_size, visibility, name, items
    // } = syn::parse_macro_input!(item as AtomicEnumInput);

    let mut input = syn::parse_macro_input!(item as DeriveInput);
    let DeriveInput {
        attrs, vis, ident, data,
        ..
    } = &mut input;

    let mut repr = Ident::new("u32", Span::call_site());
    
    attrs.retain_mut(|attr| {
        if attr.meta.path().is_ident("repr") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("u8") { repr = Ident::new("u8", Span::call_site()); }
                if meta.path.is_ident("u16") { repr = Ident::new("u16", Span::call_site()); }
                if meta.path.is_ident("u32") { repr = Ident::new("u32", Span::call_site()); }
                if meta.path.is_ident("u64") { repr = Ident::new("u64", Span::call_site()); }

                if meta.path.is_ident("i8") { repr = Ident::new("i8", Span::call_site()); }
                if meta.path.is_ident("i16") { repr = Ident::new("i16", Span::call_site()); }
                if meta.path.is_ident("i32") { repr = Ident::new("i32", Span::call_site()); }
                if meta.path.is_ident("i64") { repr = Ident::new("i64", Span::call_site()); }

                if meta.path.is_ident("usize") { repr = Ident::new("usize", Span::call_site()); }
                if meta.path.is_ident("isize") { repr = Ident::new("isize", Span::call_site()); }

                Ok(())
            });

            false 
        } else {
            true 
        }
    });

    let atomic_ident = format_ident!("Atomic{ident}");
    
    let repr_string = {
        let repr_string = repr.to_string();
        let mut chars = repr_string.chars();
        chars.next().map(|f| f.to_uppercase().collect::<String>()).unwrap_or_default() + chars.as_str()
    };

    let atomic_inner = format_ident!("Atomic{}", repr_string);
    let enum_data;

    if let Data::Enum(data) = data {
        enum_data = data;

        for variant in &enum_data.variants {
            if variant.fields != Fields::Unit {
                return TokenStream::from(quote_spanned! {
                    variant.span() => compile_error!("atomic_enum can only be applied to fieldless enums");
                })        
            }
        }
    } else {
        return TokenStream::from(quote_spanned! {
            input.span() => compile_error!("atomic_enum can only be applied to enums");
        })
    }

    let variants = &enum_data.variants;

    TokenStream::from(quote! {
        #[repr(#repr)]
        #(#attrs)*
        #vis enum #ident {
            #variants
        }

        #vis struct #atomic_ident(::std::sync::atomic::#atomic_inner);

        impl #atomic_ident {
            pub fn load(&self, ordering: ::std::sync::atomic::Ordering) -> #ident {
                let disc = self.0.load(ordering);
                unsafe {
                    ::std::mem::transmute::<#repr, #ident>(disc)
                }
            }

            pub fn store(&self, value: #ident, ordering: ::std::sync::atomic::Ordering) {
                self.0.store(value as #repr, ordering);
            }
        }

        impl From<#ident> for #atomic_ident {
            fn from(v: #ident) -> Self {
                Self(::std::sync::atomic::#atomic_inner::new(v as #repr))
            }
        }
    })

    // let atomic_ident = format_ident!("Atomic{}", name);
    // let atomic_inner = format_ident!("Atomic{}", repr.to_uppercase());

    // let expanded = quote! {
    //     #visibility struct #atomic_ident(::std::sync::atomic::#atomic_inner);

    //     impl #atomic_ident {
    //         const ENUM_SIZE: #repr = #enum_size;

    //         pub fn store(value: Self, ordering: ::std::sync::atomic::Ordering) {
    //             let disc = unsafe {
    //                 std::mem::transmute<Self, #repr>(value)
    //             };
    //             self.0.store(ordering);
    //         }

    //         #[inline]
    //         pub fn load(ordering: ::std::sync::atomic::Ordering) -> Self {
    //             let disc = self.0.load(ordering);
    //             assert!(disc < ENUM_SIZE);

    //             unsafe {
    //                 std::mem::transmute::<#repr, Self>(disc)
    //             }
    //         }
    //     }
    // };

    // TokenStream::from(expanded) 
}