use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

// #[derive(FromDeriveInput, Default)]
// #[darling(default, attributes(register_trait))]
// struct Opts {
//     address: Option<u8>,
// }

// #[proc_macro_derive(Register, attributes(register_trait))]
// pub fn derive(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input);
//     let opts = Opts::from_derive_input(&input).expect("Wrong options");
//     let DeriveInput { ident, .. } = input;
//     let output = quote! { impl Register for #ident { const ADDRESS: u8 = #answer; } };
//     output.into()
// }

#[derive(Debug, FromMeta)]
struct RegisterArgs {
    address: u8,
}

#[proc_macro_attribute]
pub fn register(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let struct_definition: proc_macro2::TokenStream = input.clone().into();

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let args = match RegisterArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let address = args.address;

    quote! {
        #struct_definition
        impl Register for #struct_name { const ADDRESS: u8 = #address; }
    }
    .into()
}
