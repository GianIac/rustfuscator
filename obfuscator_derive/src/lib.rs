use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(Obfuscate)]
pub fn derive_obfuscate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;
    let obf_name = Ident::new(&format!("Obfuscated{}", name), name.span());

    // Structs only
    let data = match &input.data {
        Data::Struct(s) => s,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[derive(Obfuscate)] can only be used on structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Named fields only
    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "Obfuscate derive only supports named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // Type validation
    for f in fields {
        let ty = &f.ty;
        if !(matches!(ty, Type::Path(p) if p.path.is_ident("String") || p.path.is_ident("u32"))) {
            return syn::Error::new_spanned(
                &f.ty,
                format!(
                    "Obfuscate derive only supports String and u32 (field `{}` has unsupported type)",
                    f.ident.as_ref().unwrap()
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    // Generated obfuscated struct fields
    let obf_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: (Vec<u8>, [u8; 12]) }
    });

    // Arguments for clear-text constructor
    let clear_args = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! { &str },
            Type::Path(p) if p.path.is_ident("u32") => quote! { u32 },
            _ => quote! { &str }, // never reached due to validation above
        };
        quote! { #name: #ty }
    });

    // Encryption in new_clear(...)
    let clear_encrypt = fields.iter().map(|f| {
        let name = &f.ident;
        match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_string(
                    #name,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("encryption failed")
            },
            Type::Path(p) if p.path.is_ident("u32") => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_u32(
                    #name,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("encryption failed")
            },
            _ => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_string(
                    #name,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("encryption failed")
            },
        }
    });

    // Decryption in get_clear()
    let decrypt_fields = fields.iter().map(|f| {
        let name = &f.ident;
        match &f.ty {
            Type::Path(p) if p.path.is_ident("String") => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_string(
                    &self.#name.0, &self.#name.1,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("decryption failed")
            },
            Type::Path(p) if p.path.is_ident("u32") => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_u32(
                    &self.#name.0, &self.#name.1,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("decryption failed")
            },
            _ => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_string(
                    &self.#name.0, &self.#name.1,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("decryption failed")
            },
        }
    });

    let expanded = quote! {
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
