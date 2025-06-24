use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

#[derive(Debug, FromMeta)]
struct RegisterArgs {
    address: u8,
    default_u8: Option<u8>,
    default_u16: Option<u16>,
    default_u24: Option<u32>,
    default_u32: Option<u32>,
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

    let RegisterArgs {
        address,
        default_u8,
        default_u16,
        default_u24,
        default_u32,
    } = match RegisterArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let (impl_const_default, impl_into_slice) = if let Some(default) = default_u8 {
        (
            quote! { impl #struct_name { pub const fn const_default() -> Self { Self(#default) } }},
            quote! {
                impl AsRef<[u8]> for #struct_name {
                    fn as_ref(&self) -> &[u8] {
                        core::slice::from_ref(&self.0)
                    }
                }
                impl AsMut<[u8]> for #struct_name {
                    fn as_mut(&mut self) -> &mut [u8] {
                        core::slice::from_mut(&mut self.0)
                    }
                }
            },
        )
    } else if let Some(default) = default_u16 {
        (
            quote! { impl #struct_name { pub const fn const_default() -> Self { Self(#default) } }},
            quote! {
                impl AsRef<[u8]> for #struct_name {
                    fn as_ref(&self) -> &[u8] {
                        let ptr = &self.0 as *const u16 as *const u8;
                        // SAFETY Self has to be repr(transparent) and u16 is 2 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts(ptr, core::mem::size_of<u16>()) }
                    }
                }
                impl AsMut<[u8]> for #struct_name {
                    fn as_mut(&mut self) -> &mut [u8] {
                        let ptr = &mut self.0 as *mut u16 as *mut u8;
                        // SAFETY Self has to be repr(transparent) and u16 is 2 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts_mut(ptr, 2) }
                    }
                }
            },
        )
    } else if let Some(default) = default_u24 {
        (
            quote! { impl #struct_name { pub const fn const_default() -> Self { Self(#default) } }},
            quote! {
                impl AsRef<[u8]> for #struct_name {
                    fn as_ref(&self) -> &[u8] {
                        let ptr = &self.0 as *const u32 as *const u8;
                        // SAFETY Self has to be repr(transparent) and u32 is more than 3 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts(ptr, 3) }
                    }
                }
                impl AsMut<[u8]> for #struct_name {
                    fn as_mut(&mut self) -> &mut [u8] {
                        let ptr = &mut self.0 as *mut u32 as *mut u8;
                        // SAFETY Self has to be repr(transparent) and u32 is more than 3 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts_mut(ptr, 3) }
                    }
                }
            },
        )
    } else if let Some(default) = default_u32 {
        (
            quote! { impl #struct_name { pub const fn const_default() -> Self { Self(#default) } }},
            quote! {
                impl AsRef<[u8]> for #struct_name {
                    fn as_ref(&self) -> &[u8] {
                        let ptr = &self.0 as *const u32 as *const u8;
                        // SAFETY Self has to be repr(transparent) and u32 is 4 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts(ptr, core::mem::size_of<u32>() + 1) }
                    }
                }
                impl AsMut<[u8]> for #struct_name {
                    fn as_mut(&mut self) -> &mut [u8] {
                        let ptr = &mut self.0 as *mut u32 as *mut u8;
                        // SAFETY Self has to be repr(transparent) and u32 is 4 bytes wide,
                        // with alignment greater than that of u8
                        unsafe { core::slice::from_raw_parts_mut(ptr, 4) }
                    }
                }
            },
        )
    } else {
        (
            syn::Error::new_spanned(struct_definition.clone(), "No default value set!")
                .into_compile_error(),
            quote! {},
        )
    };

    quote! {
        #struct_definition
        impl Register for #struct_name { const ADDRESS: u8 = #address; }
        #impl_const_default
        impl Default for #struct_name { fn default() -> Self { Self::const_default() } }
        #impl_into_slice
    }
    .into()
}
