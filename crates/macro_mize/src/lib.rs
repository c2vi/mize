use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_attribute]
pub fn mize_part(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Parse name from attribute
    let part_name = if !attr.is_empty() {
        let lit: syn::LitStr = parse_macro_input!(attr);
        lit.value()
    } else {
        struct_name.to_string().to_lowercase()
    };

    // Verify mize field exists
    let has_mize = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f
                .named
                .iter()
                .any(|field| field.ident.as_ref().map(|i| i == "mize").unwrap_or(false)),
            _ => false,
        },
        _ => false,
    };

    if !has_mize {
        return syn::Error::new_spanned(struct_name, "Struct must have a 'mize: Mize' field")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #input
        use std::any::Any;

        // impl the default new impl
        impl mize::MizePartGenerated for #struct_name {
            fn name_generated(&self) -> &'static str {
                #part_name
            }
            fn get_mize_generated(&mut self) -> &mut Mize {
              &mut self.mize
            }
            fn as_any_generated(&self) -> &dyn Any {
              self
            }
            fn as_any_mut_generated(&mut self) -> &mut dyn Any {
              self
            }
            fn into_any_generated(self: Box<Self>) -> Box<dyn Any> {
              self
            }
        }

        impl mize::MizePartCreateGenerated for #struct_name {
            fn create_generated(mize: Mize) -> Self {
                Self {
                    mize,
                    .. Default::default()
                }
            }
        }
    };

    TokenStream::from(expanded)
}
