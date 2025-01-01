use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Token, Variant,
};

#[proc_macro_derive(ErrorMeta, attributes(error_meta))]
pub fn derive_error_metadata(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let output = impl_error_metadata(&input);
  output.into()
}

fn impl_error_metadata(input: &DeriveInput) -> TokenStream2 {
  let name = &input.ident;

  match &input.data {
    Data::Enum(data) => {
      let variants = &data.variants;
      let error_meta_header = parse_enum_meta_header(&input.attrs);
      if variants.is_empty() {
        return empty_enum(name, &error_meta_header.trait_to_impl);
      }

      let error_type_body = generate_attribute_method(name, variants, "error_type");
      let code_body = generate_attribute_method(name, variants, "code");
      let args_method = generate_args_method(name, &input.data);

      generate_impl(
        name,
        error_type_body,
        code_body,
        args_method,
        &error_meta_header.trait_to_impl,
      )
    }
    Data::Struct(_) => {
      let error_meta = parse_struct_meta_attrs(&input.attrs)
        .unwrap_or_else(|| panic!("error_meta attribute missing for struct {}", name));
      let error_type = error_meta
        .error_type
        .map(
          |error_type_value| quote! { <_ as AsRef<str>>::as_ref(&#error_type_value).to_string() },
        )
        .unwrap_or_else(|| panic!("error_type attribute missing for struct {}", name));
      let code = error_meta
        .code
        .map(|code| quote! { <_ as AsRef<str>>::as_ref(&#code).to_string() })
        .unwrap_or_else(|| {
          let default_code = name.to_string().to_case(Case::Snake);
          quote! { #default_code.to_string() }
        });
      let args_method = generate_args_method(name, &input.data);
      generate_impl(
        name,
        error_type,
        code,
        args_method,
        &error_meta.trait_to_impl,
      )
    }
    Data::Union(_) => panic!("ErrorMeta can only be derived for enums and structs"),
  }
}

fn generate_impl(
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

fn empty_enum(name: &Ident, trait_to_impl: &Option<syn::Path>) -> TokenStream2 {
  generate_impl(
    name,
    quote! { unreachable!("Empty enum has no variants") },
    quote! { unreachable!("Empty enum has no variants") },
    quote! { unreachable!("Empty enum has no variants") },
    trait_to_impl,
  )
}

fn is_transparent(variant: &Variant) -> bool {
  variant.attrs.iter().any(|attr| {
    if attr.path().is_ident("error") {
      if let Ok(meta) = attr.meta.require_list() {
        if let Ok(syn::Meta::Path(path)) = meta.parse_args::<syn::Meta>() {
          return path.is_ident("transparent");
        }
      }
    }
    false
  })
}

fn generate_pattern(fields: &Fields) -> TokenStream2 {
  match fields {
    Fields::Named(_) => quote! { { .. } },
    Fields::Unnamed(_) => quote! { (..) },
    Fields::Unit => quote! {},
  }
}

#[derive(Debug, PartialEq)]
struct EnumMetaHeader {
  trait_to_impl: Option<syn::Path>,
}

impl Parse for EnumMetaHeader {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut trait_to_impl = None;

    while !input.is_empty() {
      let ident: Ident = input.parse()?;
      input.parse::<Token![=]>()?;

      if ident == "trait_to_impl" {
        trait_to_impl = Some(input.parse()?);
      } else {
        return Err(syn::Error::new(
          ident.span(),
          format!("unknown attribute '{}'", ident),
        ));
      }

      if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      }
    }

    Ok(EnumMetaHeader { trait_to_impl })
  }
}

fn parse_enum_meta_header(attrs: &[Attribute]) -> EnumMetaHeader {
  for attr in attrs {
    if attr.path().is_ident("error_meta") {
      let attrs = attr
        .parse_args::<EnumMetaHeader>()
        .unwrap_or_else(|e| panic!("error parsing error meta attrs for enum: {}", e));
      return attrs;
    }
  }
  EnumMetaHeader {
    trait_to_impl: None,
  }
}

#[derive(Debug, PartialEq)]
struct EnumMetaAttrs {
  error_type: Option<syn::Expr>,
  code: Option<syn::Expr>,
  args_delegate: Option<bool>,
}

impl Parse for EnumMetaAttrs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut error_type = None;
    let mut code = None;
    let mut args_delegate = None;

    while !input.is_empty() {
      let ident: Ident = input.parse()?;
      input.parse::<Token![=]>()?;

      match ident.to_string().as_str() {
        "error_type" => {
          error_type = Some(input.parse()?);
        }
        "code" => {
          code = Some(input.parse()?);
        }
        "args_delegate" => {
          let lit_bool: syn::LitBool = input.parse()?;
          args_delegate = Some(lit_bool.value);
        }
        attr => {
          return Err(syn::Error::new(
            ident.span(),
            format!("unknown attribute '{}'", attr),
          ))
        }
      }

      if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      }
    }

    Ok(EnumMetaAttrs {
      error_type,
      code,
      args_delegate,
    })
  }
}

fn parse_enum_meta_attrs(attrs: &[Attribute]) -> Option<EnumMetaAttrs> {
  for attr in attrs {
    if attr.path().is_ident("error_meta") {
      let attrs = attr
        .parse_args::<EnumMetaAttrs>()
        .unwrap_or_else(|e| panic!("error parsing error meta attrs for enum: {}", e));
      return Some(attrs);
    }
  }
  None
}

#[derive(Debug, PartialEq)]
struct StructMetaAttrs {
  error_type: Option<syn::Expr>,
  code: Option<syn::Expr>,
  trait_to_impl: Option<syn::Path>,
}

impl Parse for StructMetaAttrs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut error_type = None;
    let mut code = None;
    let mut trait_to_impl = None;

    while !input.is_empty() {
      let ident: Ident = input.parse()?;
      input.parse::<Token![=]>()?;

      match ident.to_string().as_str() {
        "error_type" => {
          error_type = Some(input.parse()?);
        }
        "code" => {
          code = Some(input.parse()?);
        }
        "trait_to_impl" => {
          trait_to_impl = Some(input.parse()?);
        }
        attr => {
          return Err(syn::Error::new(
            ident.span(),
            format!("unknown attribute '{}'", attr),
          ))
        }
      }

      if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
      }
    }

    Ok(StructMetaAttrs {
      error_type,
      code,
      trait_to_impl,
    })
  }
}

fn parse_struct_meta_attrs(attrs: &[Attribute]) -> Option<StructMetaAttrs> {
  for attr in attrs {
    if attr.path().is_ident("error_meta") {
      let attrs = attr
        .parse_args::<StructMetaAttrs>()
        .unwrap_or_else(|e| panic!("error parsing error meta attrs for struct: {}", e));
      return Some(attrs);
    }
  }
  None
}

fn generate_attribute_method(
  name: &Ident,
  variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
  method: &str,
) -> TokenStream2 {
  let arms = variants.iter().map(|variant| {
    let variant_name = &variant.ident;
    let pattern = generate_pattern(&variant.fields);

    let is_transparent = is_transparent(variant);
    let error_meta = parse_enum_meta_attrs(&variant.attrs);

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

fn generate_args_method(name: &Ident, data: &Data) -> TokenStream2 {
  match data {
    Data::Enum(data_enum) => {
      let arms = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let fields = &variant.fields;
        let is_transparent = is_transparent(variant);
        let error_meta = parse_enum_meta_attrs(&variant.attrs);

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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    generate_args_method, generate_attribute_method, impl_error_metadata, is_transparent,
    parse_enum_meta_attrs, EnumMetaAttrs,
  };
  use pretty_assertions::assert_eq;
  use quote::quote;
  use rstest::rstest;
  use syn::parse_quote;

  #[rstest]
  #[case(
    parse_quote!(#[error(transparent)] TransparentError),
    true
  )]
  #[case(
    parse_quote!(#[error("Some error occurred")] NonTransparentError),
    false
  )]
  #[case(
    parse_quote!(NoAttributeError),
    false
  )]
  #[case(
    parse_quote!(#[some_attr(transparent)] WithoutErrorAttribute),
    false
  )]
  fn test_is_transparent(#[case] variant: Variant, #[case] expected: bool) {
    assert_eq!(expected, is_transparent(&variant));
  }

  #[rstest]
  #[case::all_provided(
    parse_quote!(#[error_meta(error_type = "TestError", code = "test_code")]),
    Some(EnumMetaAttrs {
      error_type: Some(parse_quote!("TestError")),
      code: Some(parse_quote!("test_code")),
      args_delegate: None,
    }),
  )]
  #[case::code_fallback(
    parse_quote!(#[error_meta(error_type = "PartialError")]),
    Some(EnumMetaAttrs {
      error_type: Some(parse_quote!("PartialError")),
      code: None,
      args_delegate: None,
    }),
  )]
  #[case::as_expr(
    parse_quote!(#[error_meta(error_type = internal_server_error(), code = generate_code())]),
    Some(EnumMetaAttrs {
      error_type: Some(parse_quote!(internal_server_error())),
      code: Some(parse_quote!(generate_code())),
      args_delegate: None,
    }),
  )]
  #[case::as_enum(
    parse_quote!(#[error_meta(error_type = ErrorType::InternalServerError, code = ErrorCode::InternalServerError)]),
    Some(EnumMetaAttrs {
      error_type: Some(parse_quote!(ErrorType::InternalServerError)),
      code: Some(parse_quote!(ErrorCode::InternalServerError)),
      args_delegate: None,
    }),
  )]
  #[case::incorrect_attr(
    parse_quote!(#[other_attribute]),
    <Option<EnumMetaAttrs>>::None
  )]
  fn test_parse_error_meta(#[case] attr: Attribute, #[case] expected: Option<EnumMetaAttrs>) {
    let error_meta = parse_enum_meta_attrs(&[attr]);
    assert_eq!(expected, error_meta);
  }

  #[rstest]
  #[case::all_provided(
    parse_quote!(#[error_meta(error_type = "TestError", code = "test_code", trait_to_impl = ErrorMetadata)]),
    Some(StructMetaAttrs {
      error_type: Some(parse_quote!("TestError")),
      code: Some(parse_quote!("test_code")),
      trait_to_impl: Some(parse_quote!(ErrorMetadata)),
    }),
  )]
  #[case::minimal(
    parse_quote!(#[error_meta(error_type = "PartialError")]),
    Some(StructMetaAttrs {
      error_type: Some(parse_quote!("PartialError")),
      code: None,
      trait_to_impl: None,
    }),
  )]
  #[case::as_expr(
    parse_quote!(#[error_meta(error_type = internal_server_error(), code = generate_code(), trait_to_impl = ErrorMetadata)]),
    Some(StructMetaAttrs {
      error_type: Some(parse_quote!(internal_server_error())),
      code: Some(parse_quote!(generate_code())),
      trait_to_impl: Some(parse_quote!(ErrorMetadata)),
    }),
  )]
  #[case::as_enum(
    parse_quote!(#[error_meta(error_type = ErrorType::InternalServerError, code = ErrorCode::InternalServerError, trait_to_impl = ErrorMetadata)]),
    Some(StructMetaAttrs {
      error_type: Some(parse_quote!(ErrorType::InternalServerError)),
      code: Some(parse_quote!(ErrorCode::InternalServerError)),
      trait_to_impl: Some(parse_quote!(ErrorMetadata)),
    }),
  )]
  #[case::incorrect_attr(
    parse_quote!(#[other_attribute]),
    <Option<StructMetaAttrs>>::None
  )]
  fn test_parse_struct_error_meta(
    #[case] attr: Attribute,
    #[case] expected: Option<StructMetaAttrs>,
  ) {
    let error_meta = parse_struct_meta_attrs(&[attr]);
    assert_eq!(expected, error_meta);
  }

  #[rstest]
  #[case("error_type", quote! {
    match self {
      TestEnum::Variant1 => <_ as AsRef<str>>::as_ref(&internal_server_error()).to_string(),
      TestEnum::Variant2 => <_ as AsRef<str>>::as_ref(&"Error2").to_string(),
      TestEnum::Variant3(err) => err.error_type(),
      TestEnum::Variant4 => <_ as AsRef<str>>::as_ref(&ErrorType::InternalServerError).to_string(),
    }
  })]
  #[case("code", quote! {
    match self {
      TestEnum::Variant1 => <_ as AsRef<str>>::as_ref(&error_code()).to_string(),
      TestEnum::Variant2 => <_ as AsRef<str>>::as_ref(&"error_2").to_string(),
      TestEnum::Variant3(err) => err.code(),
      TestEnum::Variant4 => <_ as AsRef<str>>::as_ref(&ErrorCode::InternalServerError).to_string(),
    }
  })]
  fn test_generate_attribute_method_for_enum(#[case] method: &str, #[case] expected: TokenStream2) {
    let name: Ident = parse_quote!(TestEnum);
    let variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> = parse_quote! {
      #[error_meta(error_type = internal_server_error(), code = error_code())]
      Variant1,
      #[error_meta(error_type = "Error2", code = "error_2")]
      Variant2,
      #[error(transparent)]
      Variant3(std::io::Error),
      #[error_meta(error_type = ErrorType::InternalServerError, code = ErrorCode::InternalServerError)]
      Variant4
    };

    let generated = generate_attribute_method(&name, &variants, method);
    assert_eq!(expected.to_string(), generated.to_string());
  }

  #[rstest]
  fn test_generate_args_method_enum_variants() {
    let input: DeriveInput = parse_quote! {
      enum TestEnum {
        Variant1 { field1: String, field2: i32 },
        Variant2(String, i32),
        Variant3,
      }
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    assert_eq!(
      quote! {
        match self {
          TestEnum::Variant1 { field1, field2 } => {
            let mut map = ::std::collections::HashMap::new();
            map.insert(stringify!(field1).to_string(), format!("{}", field1));
            map.insert(stringify!(field2).to_string(), format!("{}", field2));
            map
          },
          TestEnum::Variant2(var_0, var_1) => {
            let mut map = ::std::collections::HashMap::new();
            map.insert(stringify!(var_0).to_string(), format!("{}", var_0));
            map.insert(stringify!(var_1).to_string(), format!("{}", var_1));
            map
          },
          TestEnum::Variant3 => ::std::collections::HashMap::new(),
        }
      }
      .to_string(),
      args_method.to_string(),
    );
  }

  fn enum_method_impls(visibility: TokenStream2) -> TokenStream2 {
    quote! {
      #visibility fn error_type(&self) -> String {
        match self {
          TestEnum::Variant1{..} => <_ as AsRef<str>>::as_ref(&"Error1").to_string(),
          TestEnum::Variant2(..) => <_ as AsRef<str>>::as_ref(&"Error2").to_string(),
        }
      }

      #visibility fn code(&self) -> String {
        match self {
          TestEnum::Variant1{..} => <_ as AsRef<str>>::as_ref(&"error_1").to_string(),
          TestEnum::Variant2(..) => "test_enum-variant_2".to_string(),
        }
      }

      #visibility fn args(&self) -> ::std::collections::HashMap<String, String> {
        match self {
          TestEnum::Variant1 { field1, field2 } => {
            let mut map = ::std::collections::HashMap::new();
            map.insert(stringify!(field1).to_string(), format!("{}", field1));
            map.insert(stringify!(field2).to_string(), format!("{}", field2));
            map
          },
          TestEnum::Variant2(var_0, var_1) => {
            let mut map = ::std::collections::HashMap::new();
            map.insert(stringify!(var_0).to_string(), format!("{}", var_0));
            map.insert(stringify!(var_1).to_string(), format!("{}", var_1));
            map
          },
        }
      }
    }
  }

  #[rstest]
  fn test_impl_error_metadata() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      enum TestEnum {
        #[error_meta(error_type = "Error1", code = "error_1")]
        Variant1 { field1: String, field2: i32 },
        #[error_meta(error_type = "Error2")]
        Variant2(String, i32),
      }
    };

    let output = impl_error_metadata(&input);
    let method_impls = enum_method_impls(quote! { pub });
    assert_eq!(
      quote! {
        impl TestEnum {
          #method_impls
        }
      }
      .to_string(),
      output.to_string(),
    );
  }

  #[test]
  fn test_impl_error_metadata_for_enum_with_trait() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      #[error_meta(trait_to_impl = ErrorMetadata)]
      enum TestEnum {
        #[error_meta(error_type = "Error1", code = "error_1")]
        Variant1 { field1: String, field2: i32 },
        #[error_meta(error_type = "Error2")]
        Variant2(String, i32),
      }
    };

    let output = impl_error_metadata(&input);
    let method_impls = enum_method_impls(quote! {});
    let expected = quote! {
      impl ErrorMetadata for TestEnum {
        #method_impls
      }
    };

    assert_eq!(expected.to_string(), output.to_string());
  }

  fn struct_method_impls(visibility: TokenStream2) -> TokenStream2 {
    quote! {
      #visibility fn error_type(&self) -> String {
        <_ as AsRef<str>>::as_ref(&"StructError").to_string()
      }

      #visibility fn code(&self) -> String {
        <_ as AsRef<str>>::as_ref(&"invalid_input").to_string()
      }

      #visibility fn args(&self) -> ::std::collections::HashMap<String, String> {
        let mut map = ::std::collections::HashMap::new();
        map.insert(stringify!(field1).to_string(), format!("{}", self.field1));
        map.insert(stringify!(field2).to_string(), format!("{}", self.field2));
        map
      }
    }
  }

  #[rstest]
  fn test_impl_error_metadata_for_struct() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      #[error_meta(error_type = "StructError", code = "invalid_input")]
      struct MyError {
        field1: String,
        field2: i32,
      }
    };
    let output = impl_error_metadata(&input);
    let method_impls = struct_method_impls(quote! { pub });
    let expected = quote! {
      impl MyError {
        #method_impls
      }
    };

    assert_eq!(expected.to_string(), output.to_string());
  }

  #[test]
  fn test_impl_error_metadata_for_struct_with_trait_to_impl() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      #[error_meta(trait_to_impl = ErrorMetadata, error_type = "StructError", code = "invalid_input")]
      struct MyError {
        field1: String,
        field2: i32,
      }
    };
    let output = impl_error_metadata(&input);
    let method_impls = struct_method_impls(quote! {});
    let expected = quote! {
      impl ErrorMetadata for MyError {
        #method_impls
      }
    };
    assert_eq!(expected.to_string(), output.to_string());
  }

  #[test]
  fn test_impl_error_metadata_for_struct_with_default_code() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      #[error_meta(error_type = "AnotherError")]
      struct AnotherError {
        message: String,
      }
    };

    let output = impl_error_metadata(&input);
    let expected = quote! {
      impl AnotherError {
        pub fn error_type(&self) -> String {
          <_ as AsRef<str>>::as_ref(&"AnotherError").to_string()
        }

        pub fn code(&self) -> String {
          "another_error".to_string()
        }

        pub fn args(&self) -> ::std::collections::HashMap<String, String> {
          let mut map = ::std::collections::HashMap::new();
          map.insert(stringify!(message).to_string(), format!("{}", self.message));
          map
        }
      }
    };

    assert_eq!(expected.to_string(), output.to_string());
  }

  #[test]
  fn test_generate_args_method_for_struct() {
    let input: DeriveInput = parse_quote! {
      struct TestStruct {
        field1: String,
        field2: i32,
      }
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    let expected = quote! {
      let mut map = ::std::collections::HashMap::new();
      map.insert(stringify!(field1).to_string(), format!("{}", self.field1));
      map.insert(stringify!(field2).to_string(), format!("{}", self.field2));
      map
    };

    assert_eq!(expected.to_string(), args_method.to_string());
  }

  #[test]
  fn test_generate_args_method_for_tuple_struct() {
    let input: DeriveInput = parse_quote! {
      struct TestTupleStruct(String, i32);
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    let expected = quote! {
      let mut map = ::std::collections::HashMap::new();
      map.insert(format!("var_{}", 0), format!("{}", self.0));
      map.insert(format!("var_{}", 1), format!("{}", self.1));
      map
    };

    assert_eq!(expected.to_string(), args_method.to_string());
  }

  #[test]
  fn test_generate_args_method_for_unit_struct() {
    let input: DeriveInput = parse_quote! {
      struct TestUnitStruct;
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    let expected = quote! {
      ::std::collections::HashMap::new()
    };

    assert_eq!(expected.to_string(), args_method.to_string());
  }

  #[rstest]
  fn test_impl_error_metadata_for_enum_with_transparent_overrides() {
    let input: DeriveInput = parse_quote! {
      #[derive(ErrorMeta)]
      enum TestEnum {
        #[error(transparent)]
        #[error_meta(error_type = "Error1", code = self.generate_code())]
        Variant1(std::io::Error)
      }
    };

    let output = impl_error_metadata(&input);
    let expected = quote! {
      impl TestEnum {
        pub fn error_type(&self) -> String {
          match self {
            TestEnum::Variant1(..) => <_ as AsRef<str>>::as_ref(&"Error1").to_string(),
          }
        }

        pub fn code(&self) -> String {
          match self {
            TestEnum::Variant1(..) => <_ as AsRef<str>>::as_ref(&self.generate_code()).to_string(),
          }
        }

        pub fn args(&self) -> ::std::collections::HashMap<String, String> {
          match self {
            TestEnum::Variant1(err) => err.args(),
          }
        }
      }
    };
    assert_eq!(expected.to_string(), output.to_string());
  }

  #[test]
  fn test_generate_args_method_with_transparent_variant() {
    let input: DeriveInput = parse_quote! {
      enum TestEnum {
        #[error(transparent)]
        Variant1(std::io::Error),
        Variant2 { field1: String, field2: i32 },
        Variant3(String, i32),
        Variant4,
      }
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    let expected = quote! {
      match self {
        TestEnum::Variant1(err) => err.args(),
        TestEnum::Variant2 { field1, field2 } => {
          let mut map = ::std::collections::HashMap::new();
          map.insert(stringify!(field1).to_string(), format!("{}", field1));
          map.insert(stringify!(field2).to_string(), format!("{}", field2));
          map
        },
        TestEnum::Variant3(var_0, var_1) => {
          let mut map = ::std::collections::HashMap::new();
          map.insert(stringify!(var_0).to_string(), format!("{}", var_0));
          map.insert(stringify!(var_1).to_string(), format!("{}", var_1));
          map
        },
        TestEnum::Variant4 => ::std::collections::HashMap::new(),
      }
    };

    assert_eq!(expected.to_string(), args_method.to_string());
  }

  #[test]
  fn test_generate_args_method_with_args_delegate() {
    let input: DeriveInput = parse_quote! {
      enum TestEnum {
        #[error(transparent)]
        #[error_meta(args_delegate = false)]
        Variant1(std::io::Error),
        #[error(transparent)]
        Variant2(OtherError),
      }
    };

    let name = &input.ident;
    let args_method = generate_args_method(name, &input.data);
    let expected = quote! {
      match self {
        TestEnum::Variant1(err) => {
          let mut map = ::std::collections::HashMap::new();
          map.insert("error".to_string(), err.to_string());
          map
        },
        TestEnum::Variant2(err) => err.args(),
      }
    };

    assert_eq!(expected.to_string(), args_method.to_string());
  }
}
