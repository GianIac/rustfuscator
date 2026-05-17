use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

const SUPPORTED_SCALARS: &[&str] = &["bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"];

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
        if !is_supported_field_type(&f.ty) {
            return syn::Error::new_spanned(
                &f.ty,
                format!(
                    "Obfuscate derive only supports String, bool, and integer scalar fields (field `{}` has unsupported type)",
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
            Type::Path(_) => {
                let ty = &f.ty;
                quote! { #ty }
            }
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
            Type::Path(_) => quote! {
                #name: rust_code_obfuscator::crypto::encrypt_display(
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
            Type::Path(_) => {
                let ty = &f.ty;
                quote! {
                    #name: rust_code_obfuscator::crypto::decrypt_parse::<#ty>(
                        &self.#name.0, &self.#name.1,
                        &rust_code_obfuscator::crypto::default_key()
                    ).expect("decryption failed")
                }
            }
            _ => quote! {
                #name: rust_code_obfuscator::crypto::decrypt_string(
                    &self.#name.0, &self.#name.1,
                    &rust_code_obfuscator::crypto::default_key()
                ).expect("decryption failed")
            },
        }
    });

    let secure_zeroize_drop = if cfg!(feature = "secure_zeroize") {
        let zeroize_fields = fields.iter().map(|f| {
            let name = &f.ident;
            quote! {
                self.#name.zeroize();
            }
        });

        quote! {
            impl ::core::ops::Drop for #name {
                fn drop(&mut self) {
                    use rust_code_obfuscator::zeroize::Zeroize;
                    #(#zeroize_fields)*
                }
            }
        }
    } else {
        quote! {}
    };

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

        #secure_zeroize_drop
    };

    TokenStream::from(expanded)
}

fn is_supported_field_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) if path.path.is_ident("String") => true,
        Type::Path(path) => SUPPORTED_SCALARS
            .iter()
            .any(|supported| path.path.is_ident(supported)),
        _ => false,
    }
}
