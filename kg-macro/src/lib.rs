use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{Attribute, Fields, ItemStruct, LitStr, Token, parse_macro_input},
};

const KG_MAPPING_DELIM: &str = " | kiro_schema_path = ";

#[proc_macro]
pub fn kg_mapping_delim(_input: TokenStream) -> TokenStream {
    let lit = KG_MAPPING_DELIM;
    TokenStream::from(quote!(#lit))
}

#[proc_macro_attribute]
pub fn kg_schema(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let mut derive_attrs: Vec<Attribute> = Vec::new();
    let mut facet_attrs: Vec<Attribute> = Vec::new();
    let mut errors: Option<syn::Error> = None;
    input.attrs.retain(|attr| {
        if attr.path().is_ident("derive") {
            derive_attrs.push(attr.clone());
            return false;
        }
        if attr.path().is_ident("facet") {
            facet_attrs.push(attr.clone());
            return false;
        }
        true
    });

    if let Fields::Named(fields) = &mut input.fields {
        for field in fields.named.iter_mut() {
            let mut has_mapping = false;
            let mut mapping_args: Option<MappingArgs> = None;
            field.attrs.retain(|attr| {
                if is_kg_mapping_attr(attr) {
                    has_mapping = true;
                    match parse_mapping_args(attr) {
                        Ok(args) => mapping_args = Some(args),
                        Err(err) => {
                            if let Some(existing) = &mut errors {
                                existing.combine(err);
                            } else {
                                errors = Some(err);
                            }
                        }
                    }
                    return false;
                }
                true
            });

            if has_mapping {
                let args = match mapping_args {
                    Some(args) => args,
                    None => continue,
                };
                let doc = format!(
                    "{}{}{}",
                    args.description, KG_MAPPING_DELIM, args.kiro_schema_path
                );
                field.attrs.retain(|attr| !is_doc_attr(attr));
                field.attrs.push(syn::parse_quote!(#[doc = #doc]));
            }
        }
    }

    if derive_attrs.is_empty() {
        derive_attrs.push(syn::parse_quote!(#[derive(Facet, Clone, Default)]));
    }

    if facet_attrs.is_empty() {
        facet_attrs.push(syn::parse_quote!(
            #[facet(deny_unknown_fields, skip_all_unless_truthy, default)]
        ));
    }

    if let Some(err) = errors {
        return TokenStream::from(err.to_compile_error());
    }

    TokenStream::from(quote!(
        #(#derive_attrs)*
        #(#facet_attrs)*
        #input
    ))
}

fn is_kg_mapping_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("kg_mapping")
}

fn is_doc_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("doc")
}

struct MappingArgs {
    kiro_schema_path: String,
    description: String,
}

fn parse_mapping_args(attr: &Attribute) -> syn::Result<MappingArgs> {
    let mut kiro_schema_path: Option<String> = None;
    let mut description: Option<String> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("kiro_schema_path") {
            if kiro_schema_path.is_some() {
                return Err(meta.error("duplicate kiro_schema_path"));
            }
            let lit: LitStr = meta.value()?.parse()?;
            kiro_schema_path = Some(lit.value());
            return Ok(());
        }

        if meta.path.is_ident("description") {
            if description.is_some() {
                return Err(meta.error("duplicate description"));
            }
            let lit: LitStr = meta.value()?.parse()?;
            description = Some(lit.value());
            return Ok(());
        }

        if meta.input.peek(Token![=]) {
            let _ = meta.value()?.parse::<syn::Expr>()?;
            return Err(
                meta.error("unknown kg_mapping argument, expected kiro_schema_path or description")
            );
        }

        if meta.input.peek(syn::token::Paren) {
            return Err(
                meta.error("unknown kg_mapping argument, expected kiro_schema_path or description")
            );
        }

        Err(meta.error("unknown kg_mapping argument, expected kiro_schema_path or description"))
    })?;

    let kiro_schema_path = kiro_schema_path.ok_or_else(|| {
        syn::Error::new_spanned(
            attr,
            "missing required kg_mapping argument: kiro_schema_path",
        )
    })?;
    let description = description.ok_or_else(|| {
        syn::Error::new_spanned(attr, "missing required kg_mapping argument: description")
    })?;

    Ok(MappingArgs {
        kiro_schema_path,
        description,
    })
}
