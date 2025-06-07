use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(Obfuscate)]
pub fn derive_obfuscate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;
    let obf_name = Ident::new(&format!("Obfuscated{}", name), name.span());

    let data = match &input.data {
        Data::Struct(s) => s,
        _ => panic!("Obfuscate can only be derived for structs"),
    };

    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("Only named fields are supported"),
    };

    let obf_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: (Vec<u8>, [u8; 12])
        }
    });

    let clear_args = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! { &str },
            Type::Path(p) if p.path.is_ident("u32") => quote! { u32 },
            _ => quote! { &str },
        };
        quote! { #name: #ty }
    });

    let clear_encrypt = fields.iter().map(|f| {
        let name = &f.ident;
        match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_string(#name, AES_KEY)
            },
            Type::Path(p) if p.path.is_ident("u32") => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_u32(#name, AES_KEY)
            },
            _ => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_string(#name, AES_KEY)
            },
        }
    });

    let decrypt_fields = fields.iter().map(|f| {
        let name = &f.ident;
        match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_string(&self.#name.0, &self.#name.1, AES_KEY)
            },
            Type::Path(p) if p.path.is_ident("u32") => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_u32(&self.#name.0, &self.#name.1, AES_KEY)
            },
            _ => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_string(&self.#name.0, &self.#name.1, AES_KEY)
            },
        }
    });

    let expanded = quote! {
        use rust_code_obfuscator::crypto::{AES_KEY};

        #[derive(Clone)]
        #vis struct #obf_name {
            #(#obf_fields),*
        }

        impl #obf_name {
            pub fn new_clear(#(#clear_args),*) -> Self {
                Self {
                    #(#clear_encrypt),*
                }
            }

            pub fn get_clear(&self) -> #name {
                #name {
                    #(#decrypt_fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
