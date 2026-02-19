mod generate;
mod parse;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

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
      let error_meta_header = parse::parse_enum_meta_header(&input.attrs);
      if variants.is_empty() {
        return generate::empty_enum(name, &error_meta_header.trait_to_impl);
      }

      let error_type_body = generate::generate_attribute_method(name, variants, "error_type");
      let code_body = generate::generate_attribute_method(name, variants, "code");
      let args_method = generate::generate_args_method(name, &input.data);

      generate::generate_impl(
        name,
        error_type_body,
        code_body,
        args_method,
        &error_meta_header.trait_to_impl,
      )
    }
    Data::Struct(_) => {
      let error_meta = parse::parse_struct_meta_attrs(&input.attrs)
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
      let args_method = generate::generate_args_method(name, &input.data);
      generate::generate_impl(
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

#[cfg(test)]
mod tests {
  use crate::generate::{generate_args_method, generate_attribute_method};
  use crate::parse::{
    parse_enum_meta_attrs, parse_struct_meta_attrs, EnumMetaAttrs, StructMetaAttrs,
  };
  use crate::{impl_error_metadata, parse};
  use pretty_assertions::assert_eq;
  use proc_macro2::TokenStream as TokenStream2;
  use quote::quote;
  use rstest::rstest;
  use syn::parse_quote;
  use syn::{Attribute, DeriveInput, Ident, Variant};

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
    assert_eq!(expected, parse::is_transparent(&variant));
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
