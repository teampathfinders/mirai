use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Token, Visibility};

struct CodecInput {
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
}

impl Parse for CodecInput {
    fn parse(input: ParseStream) -> Result<CodecInput> {
        let vis: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident: Ident = input.parse()?;

        input.parse::<Token![<]>()?;
        let generics: Generics = input.parse()?;
        input.parse::<Token![>]>()?;

        Ok(CodecInput {
            vis, ident, generics
        })
    }
}

// #[derive(Codec)]
// struct Packet {
//     #[le]
//     little_endian: i32,
//     #[be]
//     big_endian: i32,
//     #[var]
//     var_endian: i32,
//     #[nbt]
//     nbt_data: SomeStruct
// }

fn build_ser_struct(data: &DataStruct) -> TokenStream {
    let fields_iter = match data.fields {
        Fields::Named(ref fields) => Some(fields.named.iter()),
        Fields::Unnamed(ref fields) => Some(fields.unnamed.iter()),
        Fields::Unit => None
    };



    todo!()
}

pub fn inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = &input.vis;
    let ident = &input.ident;

    print!("{input:?}");

    // let (ser, de) = match input.data {
    //     Data::Struct(v) => (
    //         build_ser_struct(&v), build_de_struct(v)
    //     ),
    //     Data::Enum(v) => (
    //         build_ser_enum(&v), build_de_enum(v)
    //     ),
    //     Data::Union(_) => return TokenStream::from(quote! {
    //         compile_error!("Codec derive macro only supports structs and enums")
    //     })
    // };

    TokenStream::from(match input.data {
        Data::Struct(ref data) => {
            quote! {
                #vis struct #ident {

                }

                impl CodecSerialize for #ident {

                }
            }
        },
        _ => todo!()
    })
}
