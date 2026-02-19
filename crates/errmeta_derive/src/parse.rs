use syn::{
  parse::{Parse, ParseStream},
  Attribute, Ident, Token, Variant,
};

#[derive(Debug, PartialEq)]
pub struct EnumMetaHeader {
  pub trait_to_impl: Option<syn::Path>,
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

pub fn parse_enum_meta_header(attrs: &[Attribute]) -> EnumMetaHeader {
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
pub struct EnumMetaAttrs {
  pub error_type: Option<syn::Expr>,
  pub code: Option<syn::Expr>,
  pub args_delegate: Option<bool>,
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

pub fn parse_enum_meta_attrs(attrs: &[Attribute]) -> Option<EnumMetaAttrs> {
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
pub struct StructMetaAttrs {
  pub error_type: Option<syn::Expr>,
  pub code: Option<syn::Expr>,
  pub trait_to_impl: Option<syn::Path>,
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

pub fn parse_struct_meta_attrs(attrs: &[Attribute]) -> Option<StructMetaAttrs> {
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

pub fn is_transparent(variant: &Variant) -> bool {
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
