use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, Fields, Ident};

use crate::parse::{self, EnumMetaAttrs};

pub fn generate_impl(
  name: &Ident,
  error_type_body: TokenStream2,
  code_body: TokenStream2,
  args_method: TokenStream2,
  trait_to_impl: &Option<syn::Path>,
) -> TokenStream2 {
  let visibility = if trait_to_impl.is_some() {
    quote! {}
  } else {
    quote! { pub }
  };

  let impl_block = quote! {
    #visibility fn error_type(&self) -> String {
      #error_type_body
    }

    #visibility fn code(&self) -> String {
      #code_body
    }

    #visibility fn args(&self) -> ::std::collections::HashMap<String, String> {
      #args_method
    }
  };

  match trait_to_impl {
    Some(trait_path) => quote! {
      impl #trait_path for #name {
        #impl_block
      }
    },
    None => quote! {
      impl #name {
        #impl_block
      }
    },
  }
}

pub fn empty_enum(name: &Ident, trait_to_impl: &Option<syn::Path>) -> TokenStream2 {
  generate_impl(
    name,
    quote! { unreachable!("Empty enum has no variants") },
    quote! { unreachable!("Empty enum has no variants") },
    quote! { unreachable!("Empty enum has no variants") },
    trait_to_impl,
  )
}

fn generate_pattern(fields: &Fields) -> TokenStream2 {
  match fields {
    Fields::Named(_) => quote! { { .. } },
    Fields::Unnamed(_) => quote! { (..) },
    Fields::Unit => quote! {},
  }
}

pub fn generate_attribute_method(
  name: &Ident,
  variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
  method: &str,
) -> TokenStream2 {
  let arms = variants.iter().map(|variant| {
    let variant_name = &variant.ident;
    let pattern = generate_pattern(&variant.fields);

    let is_transparent = parse::is_transparent(variant);
    let error_meta = parse::parse_enum_meta_attrs(&variant.attrs);

    match method {
      "error_type" => {
        generate_error_type_arm(name, variant_name, &pattern, is_transparent, &error_meta)
      }
      "code" => generate_code_arm(name, variant_name, &pattern, is_transparent, &error_meta),
      _ => unreachable!(),
    }
  });

  quote! {
    match self {
      #(#arms)*
    }
  }
}

fn generate_error_type_arm(
  name: &Ident,
  variant_name: &Ident,
  pattern: &TokenStream2,
  is_transparent: bool,
  error_meta: &Option<EnumMetaAttrs>,
) -> TokenStream2 {
  if let Some(error_meta) = error_meta {
    if let Some(error_type) = &error_meta.error_type {
      return quote! {
        #name::#variant_name #pattern => <_ as AsRef<str>>::as_ref(&#error_type).to_string(),
      };
    }
  }

  if is_transparent {
    quote! {
      #name::#variant_name(err) => err.error_type(),
    }
  } else {
    let msg = format!("error_type not specified for variant '{}'", variant_name);
    quote! {
      #name::#variant_name #pattern => compile_error!(#msg),
    }
  }
}

fn generate_code_arm(
  name: &Ident,
  variant_name: &Ident,
  pattern: &TokenStream2,
  is_transparent: bool,
  error_meta: &Option<EnumMetaAttrs>,
) -> TokenStream2 {
  if let Some(error_meta) = error_meta {
    if let Some(code) = &error_meta.code {
      return quote! {
        #name::#variant_name #pattern => <_ as AsRef<str>>::as_ref(&#code).to_string(),
      };
    }
  }

  if is_transparent {
    quote! {
      #name::#variant_name(err) => err.code(),
    }
  } else {
    let default_code = format!(
      "{}-{}",
      name.to_string().to_case(Case::Snake),
      variant_name.to_string().to_case(Case::Snake)
    );
    quote! {
      #name::#variant_name #pattern => #default_code.to_string(),
    }
  }
}

pub fn generate_args_method(name: &Ident, data: &Data) -> TokenStream2 {
  match data {
    Data::Enum(data_enum) => {
      let arms = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let fields = &variant.fields;
        let is_transparent = parse::is_transparent(variant);
        let error_meta = parse::parse_enum_meta_attrs(&variant.attrs);

        if is_transparent {
          let args_delegate = error_meta.and_then(|meta| meta.args_delegate).unwrap_or(true);

          if args_delegate {
            quote! {
              #name::#variant_name(err) => err.args()
            }
          } else {
            quote! {
              #name::#variant_name(err) => {
                let mut map = ::std::collections::HashMap::new();
                map.insert("error".to_string(), err.to_string());
                map
              }
            }
          }
        } else {
          match fields {
            Fields::Named(named_fields) => {
              let field_names_splat = named_fields.named.iter().map(|f| &f.ident);
              let field_names = named_fields.named.iter().map(|f| &f.ident);
              quote! {
                #name::#variant_name { #(#field_names_splat),* } => {
                  let mut map = ::std::collections::HashMap::new();
                  #(
                    map.insert(stringify!(#field_names).to_string(), format!("{}", #field_names));
                  )*
                  map
                }
              }
            }
            Fields::Unnamed(unnamed_fields) => {
              let field_indices_names: Vec<_> = (0..unnamed_fields.unnamed.len())
                .map(|i| format_ident!("var_{i}"))
                .collect();
              quote! {
                #name::#variant_name(#(#field_indices_names),*) => {
                  let mut map = ::std::collections::HashMap::new();
                  #(
                    map.insert(stringify!(#field_indices_names).to_string(), format!("{}", #field_indices_names));
                  )*
                  map
                }
              }
            }
            Fields::Unit => {
              quote! {
                #name::#variant_name => ::std::collections::HashMap::new()
              }
            }
          }
        }
      });

      quote! {
        match self {
          #(#arms,)*
        }
      }
    }
    Data::Struct(data_struct) => {
      let fields = &data_struct.fields;

      match fields {
        Fields::Named(named_fields) => {
          let field_names = named_fields.named.iter().map(|f| &f.ident);
          quote! {
            let mut map = ::std::collections::HashMap::new();
            #(
              map.insert(stringify!(#field_names).to_string(), format!("{}", self.#field_names));
            )*
            map
          }
        }
        Fields::Unnamed(unnamed_fields) => {
          let field_indices: Vec<_> = (0..unnamed_fields.unnamed.len())
            .map(syn::Index::from)
            .collect();
          quote! {
            let mut map = ::std::collections::HashMap::new();
            #(
              map.insert(format!("var_{}", #field_indices), format!("{}", self.#field_indices));
            )*
            map
          }
        }
        Fields::Unit => {
          quote! {
            ::std::collections::HashMap::new()
          }
        }
      }
    }
    Data::Union(_) => panic!("Unions are not supported"),
  }
}
