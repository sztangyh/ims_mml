use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Field, Fields, GenericArgument, Ident,
    Lifetime, LitStr, Result, Token, Type, TypePath, Variant,
};

fn parse_lifetime_opt(input: ParseStream<'_>) -> Result<Option<Lifetime>> {
    if input.peek(Token![<]) {
        let _lt_tok: Token![<] = input.parse()?;
        let lt: Lifetime = input.parse()?;
        let _gt_tok: Token![>] = input.parse()?;
        Ok(Some(lt))
    } else {
        Ok(None)
    }
}

fn option_inner(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    }
}

enum DefFieldKind {
    Ignore,
    Absent,
    Mapped,
    Optional(Type),
    Required(Type),
}

struct DefField {
    name: Ident,
    kind: DefFieldKind,
}

impl Parse for DefField {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;

        if input.peek(Ident) {
            let fork = input.fork();
            let marker: Ident = fork.parse()?;
            if marker == "Ignore" {
                let _real: Ident = input.parse()?;
                return Ok(Self {
                    name,
                    kind: DefFieldKind::Ignore,
                });
            }
            if marker == "Absent" {
                let _real: Ident = input.parse()?;
                return Ok(Self {
                    name,
                    kind: DefFieldKind::Absent,
                });
            }
            if marker == "Mapped" {
                let _real: Ident = input.parse()?;
                return Ok(Self {
                    name,
                    kind: DefFieldKind::Mapped,
                });
            }
        }

        let ty: Type = input.parse()?;
        if let Some(inner) = option_inner(&ty) {
            Ok(Self {
                name,
                kind: DefFieldKind::Optional(inner),
            })
        } else {
            Ok(Self {
                name,
                kind: DefFieldKind::Required(ty),
            })
        }
    }
}

struct MmlDefInput {
    builder: Ident,
    mml: Ident,
    lifetime: Option<Lifetime>,
    fields: Punctuated<DefField, Token![,]>,
}

impl Parse for MmlDefInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let builder: Ident = input.parse()?;
        let _comma1: Token![,] = input.parse()?;
        let mml: Ident = input.parse()?;
        let lifetime = parse_lifetime_opt(input)?;
        let _comma2: Token![,] = input.parse()?;
        let fields = Punctuated::<DefField, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            builder,
            mml,
            lifetime,
            fields,
        })
    }
}

#[proc_macro]
pub fn mml_def(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MmlDefInput);

    let builder = input.builder;
    let mml = input.mml;
    let lifetime = input.lifetime;

    let mut required = Vec::<(Ident, Type)>::new();
    let mut optional = Vec::<(Ident, Type)>::new();
    let mut absent = Vec::<Ident>::new();
    let mut ignored = Vec::<Ident>::new();
    let mut mapped = Vec::<Ident>::new();

    for f in input.fields {
        match f.kind {
            DefFieldKind::Ignore => ignored.push(f.name),
            DefFieldKind::Absent => absent.push(f.name),
            DefFieldKind::Mapped => mapped.push(f.name),
            DefFieldKind::Optional(ty) => optional.push((f.name, ty)),
            DefFieldKind::Required(ty) => required.push((f.name, ty)),
        }
    }

    let lt_decl = if let Some(lt) = &lifetime {
        quote!(<#lt>)
    } else {
        quote!()
    };
    let body_ty = if let Some(lt) = &lifetime {
        quote!(&#lt str)
    } else {
        quote!(&str)
    };
    let hm_ty = if let Some(lt) = &lifetime {
        quote!(std::collections::BTreeMap<&#lt str, &#lt str>)
    } else {
        quote!(std::collections::BTreeMap<String, String>)
    };

    let req_names: Vec<_> = required.iter().map(|(n, _)| n).collect();
    let req_tys: Vec<_> = required.iter().map(|(_, t)| t).collect();
    let opt_names: Vec<_> = optional.iter().map(|(n, _)| n).collect();
    let opt_tys: Vec<_> = optional.iter().map(|(_, t)| t).collect();

    let mapper_arg = if mapped.is_empty() {
        quote!()
    } else {
        quote!(mapper: &mut std::collections::HashMap<std::rc::Rc<str>, std::rc::Rc<str>>,)
    };
    let mapper_bind = if mapped.is_empty() {
        quote!()
    } else {
        quote!(let mapper = mapper;)
    };

    let expanded = quote! {
        #[derive(Debug, Clone)]
        #[allow(unused)]
        #[allow(non_camel_case_types)]
        pub struct #builder {
            #(
                pub #absent: String,
            )*
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub struct #mml #lt_decl {
            #(
                pub #req_names: #req_tys,
            )*
            #(
                pub #opt_names: Option<#opt_tys>,
            )*
            #(
                pub #mapped: std::rc::Rc<str>,
            )*
            pub base: std::rc::Rc<#builder>,
            pub differs: #hm_ty,
        }

        impl #lt_decl #builder {
            #[allow(unused)]
            pub fn new(hm: &std::collections::HashMap<String, String>) -> Self {
                #(
                    let #absent = hm.get(stringify!(#absent)).unwrap().clone();
                )*
                Self {
                    #(#absent,)*
                }
            }

            #[allow(unused)]
            pub fn from_mml_body(
                self: &std::rc::Rc<Self>,
                #mapper_arg
                body: #body_ty,
            ) -> Result<#mml #lt_decl, ParaParseError> {
                #mapper_bind
                #(
                    let mut #req_names: Option<#req_tys> = None;
                )*
                #(
                    let mut #opt_names: Option<#opt_tys> = None;
                )*
                #(
                    let mut #absent: Option<&str> = None;
                )*
                #(
                    let mut #mapped: Option<std::rc::Rc<str>> = None;
                )*

                let mut differ = std::collections::BTreeMap::new();
                let mut paras = mml_paras_iter(body);
                while let Some((pname, pval)) = paras.next() {
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#req_names)) {
                            let v = <#req_tys>::from_para(pval).map_err(|_e| {
                                ParaParseError::InvalidFormat(
                                    format!("{}: {}", stringify!(#req_names), pval),
                                )
                            })?;
                            #req_names = Some(v);
                        } else
                    )*
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#opt_names)) {
                            let v = <#opt_tys>::from_para(pval).map_err(|_e| {
                                ParaParseError::InvalidFormat(
                                    format!("{}: {}", stringify!(#opt_names), pval),
                                )
                            })?;
                            #opt_names = Some(v);
                        } else
                    )*
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#absent)) {
                            #absent = Some(pval);
                        } else
                    )*
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#mapped)) {
                            if let Some((_k, v)) = mapper.get_key_value(pval) {
                                #mapped = Some(v.clone());
                            } else {
                                let v: std::rc::Rc<str> = std::rc::Rc::from(pval);
                                mapper.insert(v.clone(), v.clone());
                                #mapped = Some(v);
                            }
                        } else
                    )*
                    if #( !pname.eq_ignore_ascii_case(stringify!(#ignored)) && )* true {
                        differ.insert(pname.into(), pval.into());
                    }
                }

                #(
                    if let Some(s) = #absent {
                        if s != self.#absent.as_str() {
                            differ.insert(stringify!(#absent).into(), s.into());
                        }
                    } else {
                        differ.insert(stringify!(#absent).into(), "".into());
                    }
                )*

                Ok(#mml {
                    #(
                        #req_names: #req_names.ok_or(ParaParseError::NotFound(stringify!(#req_names).into()))?,
                    )*
                    #(
                        #opt_names: #opt_names,
                    )*
                    #(
                        #mapped: #mapped.ok_or(ParaParseError::NotFound(stringify!(#mapped).into()))?,
                    )*
                    base: self.clone(),
                    differs: differ,
                })
            }
        }
    };

    expanded.into()
}

enum EasyFieldKind {
    Required(Type),
    Optional(Type),
}

enum EasyEntry {
    Cond { name: Ident, expr: Expr },
    Field { name: Ident, kind: EasyFieldKind },
}

impl Parse for EasyEntry {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            return Ok(Self::Cond { name, expr });
        }

        let _colon: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;
        if let Some(inner) = option_inner(&ty) {
            Ok(Self::Field {
                name,
                kind: EasyFieldKind::Optional(inner),
            })
        } else {
            Ok(Self::Field {
                name,
                kind: EasyFieldKind::Required(ty),
            })
        }
    }
}

struct MmlEasyInput {
    mml: Ident,
    lifetime: Option<Lifetime>,
    entries: Punctuated<EasyEntry, Token![,]>,
}

impl Parse for MmlEasyInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mml: Ident = input.parse()?;
        let lifetime = parse_lifetime_opt(input)?;
        let _comma: Token![,] = input.parse()?;
        let entries = Punctuated::<EasyEntry, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            mml,
            lifetime,
            entries,
        })
    }
}

#[proc_macro]
pub fn mml_easy(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MmlEasyInput);

    let mml = input.mml;
    let lifetime = input.lifetime;

    let mut required = Vec::<(Ident, Type)>::new();
    let mut optional = Vec::<(Ident, Type)>::new();
    let mut conds = Vec::<(Ident, Expr)>::new();

    for e in input.entries {
        match e {
            EasyEntry::Cond { name, expr } => conds.push((name, expr)),
            EasyEntry::Field { name, kind } => match kind {
                EasyFieldKind::Required(ty) => required.push((name, ty)),
                EasyFieldKind::Optional(ty) => optional.push((name, ty)),
            },
        }
    }

    let lt_decl = if let Some(lt) = &lifetime {
        quote!(<#lt>)
    } else {
        quote!()
    };
    let body_ty = if let Some(lt) = &lifetime {
        quote!(&#lt str)
    } else {
        quote!(&str)
    };

    let req_names: Vec<_> = required.iter().map(|(n, _)| n).collect();
    let req_tys: Vec<_> = required.iter().map(|(_, t)| t).collect();
    let opt_names: Vec<_> = optional.iter().map(|(n, _)| n).collect();
    let opt_tys: Vec<_> = optional.iter().map(|(_, t)| t).collect();
    let cond_names: Vec<_> = conds.iter().map(|(n, _)| n).collect();
    let cond_exprs: Vec<_> = conds.iter().map(|(_, e)| e).collect();

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub struct #mml #lt_decl {
            #(
                pub #req_names: #req_tys,
            )*
            #(
                pub #opt_names: Option<#opt_tys>,
            )*
        }

        impl #lt_decl #mml #lt_decl {
            #[allow(unused)]
            pub fn from_mml_body(body: #body_ty) -> Option<Self> {
                #(
                    let mut #req_names: Option<#req_tys> = None;
                )*
                #(
                    let mut #opt_names: Option<#opt_tys> = None;
                )*
                #(
                    let mut #cond_names = false;
                )*

                let mut paras = mml_paras_iter(body);
                while let Some((pname, pval)) = paras.next() {
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#cond_names)) {
                            #cond_names = true;
                            if pval != #cond_exprs {
                                return None;
                            }
                        } else
                    )*
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#req_names)) {
                            #req_names = Some(<#req_tys>::from_para(pval).ok()?);
                        } else
                    )*
                    #(
                        if pname.eq_ignore_ascii_case(stringify!(#opt_names)) {
                            #opt_names = Some(<#opt_tys>::from_para(pval).ok()?);
                        } else
                    )*
                    {
                        continue;
                    }
                }

                let chk = #(#cond_names &&)* true;
                if !chk {
                    return None;
                }

                Some(#mml {
                    #(
                        #req_names: #req_names?,
                    )*
                    #(
                        #opt_names: #opt_names,
                    )*
                })
            }
        }
    };

    expanded.into()
}

struct EnumInput {
    name: Ident,
    vars: Punctuated<Ident, Token![,]>,
}

impl Parse for EnumInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Self { name, vars })
    }
}

#[proc_macro]
pub fn mml_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EnumInput);
    let name = input.name;
    let vars: Vec<_> = input.vars.into_iter().collect();
    let enum_variants = vars.iter().map(|v| quote!(#v,));
    let from_str_arms = vars.iter().map(|v| quote!(stringify!(#v) => Ok(Self::#v),));
    let from_para_ifs = vars.iter().map(|v| {
        quote!(
            if s.eq_ignore_ascii_case(stringify!(#v)) {
                Ok(Self::#v)
            } else
        )
    });

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum #name {
            #(#enum_variants)*
        }

        impl std::str::FromStr for #name {
            type Err = InvalidEnumValue;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms)*
                    _ => Err(InvalidEnumValue),
                }
            }
        }

        impl<'a> FromPara<'a> for #name {
            type Err = InvalidEnumValue;

            fn from_para(s: &'a str) -> Result<Self, Self::Err> {
                #(#from_para_ifs)*
                {
                    Err(InvalidEnumValue)
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };

    expanded.into()
}

#[derive(Default)]
struct MmlFieldAttrs {
    rename: Option<LitStr>,
    skip: bool,
}

#[derive(Default)]
struct MmlStructAttrs {
    op: Option<LitStr>,
    object: Option<LitStr>,
}

#[derive(Default)]
struct MmlEnumAttrs {
    tag: Option<LitStr>,
}

#[derive(Default)]
struct MmlVariantAttrs {
    rename: Option<LitStr>,
}

fn parse_mml_field_attrs(field: &Field) -> Result<MmlFieldAttrs> {
    let mut cfg = MmlFieldAttrs::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("mml") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                cfg.rename = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            if meta.path.is_ident("skip") {
                cfg.skip = true;
                return Ok(());
            }
            Err(meta.error("unsupported field attribute, use rename/skip"))
        })?;
    }
    Ok(cfg)
}

fn parse_mml_struct_attrs(attrs: &[Attribute]) -> Result<MmlStructAttrs> {
    let mut cfg = MmlStructAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("mml") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("op") {
                cfg.op = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            if meta.path.is_ident("object") {
                cfg.object = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            Err(meta.error("unsupported struct attribute, use op/object"))
        })?;
    }
    Ok(cfg)
}

fn parse_mml_enum_attrs(attrs: &[Attribute]) -> Result<MmlEnumAttrs> {
    let mut cfg = MmlEnumAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("mml") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                cfg.tag = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            Err(meta.error("unsupported enum attribute, use tag"))
        })?;
    }
    Ok(cfg)
}

fn parse_mml_variant_attrs(variant: &Variant) -> Result<MmlVariantAttrs> {
    let mut cfg = MmlVariantAttrs::default();
    for attr in &variant.attrs {
        if !attr.path().is_ident("mml") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                cfg.rename = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            Err(meta.error("unsupported variant attribute, use rename"))
        })?;
    }
    Ok(cfg)
}

#[proc_macro_derive(MmlMessage, attributes(mml))]
pub fn derive_mml_message(input: TokenStream) -> TokenStream {
    derive_mml_message_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive_mml_message_impl(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let attrs = parse_mml_struct_attrs(&input.attrs)?;
    let op = attrs.op.unwrap_or_else(|| LitStr::new("", input.span()));
    let object = attrs
        .object
        .unwrap_or_else(|| LitStr::new("", input.span()));

    let name = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => {
                return Err(syn::Error::new(
                    data.fields.span(),
                    "MmlMessage supports only structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                name.span(),
                "MmlMessage can only be derived for struct",
            ))
        }
    };

    let mut deser_steps = Vec::new();
    let mut ser_steps = Vec::new();
    let mut build_fields = Vec::new();

    for field in fields {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "named field expected"))?;
        let cfg = parse_mml_field_attrs(&field)?;

        if cfg.skip {
            deser_steps.push(quote! {
                let #ident = ::core::default::Default::default();
            });
            build_fields.push(quote! { #ident });
            continue;
        }

        let pname = cfg.rename.unwrap_or_else(|| {
            let key = ident.to_string().to_ascii_uppercase();
            LitStr::new(&key, ident.span())
        });

        if let Some(inner_ty) = option_inner(&field.ty) {
            deser_steps.push(quote! {
                let #ident = if <#inner_ty as ::mml_def::MmlField>::has_field(#pname, params) {
                    Some(<#inner_ty as ::mml_def::MmlField>::from_mml_field(#pname, params)?)
                } else {
                    None
                };
            });
            ser_steps.push(quote! {
                if let Some(value) = &self.#ident {
                    <#inner_ty as ::mml_def::MmlField>::append_mml_field(value, #pname, &mut out)?;
                }
            });
        } else {
            let ty = field.ty;
            deser_steps.push(quote! {
                let #ident = <#ty as ::mml_def::MmlField>::from_mml_field(#pname, params)?;
            });
            ser_steps.push(quote! {
                <#ty as ::mml_def::MmlField>::append_mml_field(&self.#ident, #pname, &mut out)?;
            });
        }

        build_fields.push(quote! { #ident });
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::mml_def::MmlDeserialize for #name #ty_generics #where_clause {
            fn from_mml_params(params: &::mml_def::MmlParams) -> Result<Self, ::mml_def::MmlError> {
                #(#deser_steps)*
                Ok(Self {
                    #(#build_fields,)*
                })
            }
        }

        impl #impl_generics ::mml_def::MmlSerialize for #name #ty_generics #where_clause {
            fn to_mml_params(&self) -> Result<Vec<(String, String)>, ::mml_def::MmlError> {
                let mut out = Vec::new();
                #(#ser_steps)*
                Ok(out)
            }
        }

        impl #impl_generics ::mml_def::MmlCommand for #name #ty_generics #where_clause {
            const MML_OP: &'static str = #op;
            const MML_OBJECT: &'static str = #object;
        }
    })
}

#[proc_macro_derive(MmlBranch, attributes(mml))]
pub fn derive_mml_branch(input: TokenStream) -> TokenStream {
    derive_mml_branch_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive_mml_branch_impl(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let enum_attrs = parse_mml_enum_attrs(&input.attrs)?;
    let tag_const = if let Some(tag) = enum_attrs.tag {
        quote! { Some(#tag) }
    } else {
        quote! { None }
    };

    let name = input.ident;
    let generics = input.generics;

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return Err(syn::Error::new(
                name.span(),
                "MmlBranch can only be derived for enum",
            ))
        }
    };

    let mut parse_checks = Vec::new();
    let mut serialize_arms = Vec::new();

    for variant in variants {
        let vcfg = parse_mml_variant_attrs(&variant)?;
        let vname = variant.ident.clone();
        let tag_value = vcfg.rename.unwrap_or_else(|| {
            let tag = vname.to_string().to_ascii_uppercase();
            LitStr::new(&tag, vname.span())
        });

        match variant.fields {
            Fields::Unit => {
                parse_checks.push(quote! {
                    if tag_value.eq_ignore_ascii_case(#tag_value) {
                        return Ok(Self::#vname);
                    }
                });

                serialize_arms.push(quote! {
                    Self::#vname => {
                        out.push((actual_tag_key.to_string(), #tag_value.to_string()));
                        Ok(())
                    }
                });
            }
            Fields::Named(named) => {
                let mut parse_steps = Vec::new();
                let mut build_parts = Vec::new();

                let mut arm_bindings = Vec::new();
                let mut arm_steps = Vec::new();

                for field in named.named {
                    let fname = field
                        .ident
                        .clone()
                        .ok_or_else(|| syn::Error::new(field.span(), "named field expected"))?;
                    let fcfg = parse_mml_field_attrs(&field)?;

                    if fcfg.skip {
                        parse_steps.push(quote! {
                            let #fname = ::core::default::Default::default();
                        });
                        build_parts.push(quote! { #fname: #fname });
                        continue;
                    }

                    let pname = fcfg.rename.unwrap_or_else(|| {
                        let key = fname.to_string().to_ascii_uppercase();
                        LitStr::new(&key, fname.span())
                    });

                    if let Some(inner_ty) = option_inner(&field.ty) {
                        parse_steps.push(quote! {
                            let #fname = if <#inner_ty as ::mml_def::MmlField>::has_field(#pname, params) {
                                Some(<#inner_ty as ::mml_def::MmlField>::from_mml_field(#pname, params)?)
                            } else {
                                None
                            };
                        });
                        arm_bindings.push(quote! { #fname });
                        arm_steps.push(quote! {
                            if let Some(value) = #fname {
                                <#inner_ty as ::mml_def::MmlField>::append_mml_field(value, #pname, out)?;
                            }
                        });
                    } else {
                        let fty = field.ty;
                        parse_steps.push(quote! {
                            let #fname = <#fty as ::mml_def::MmlField>::from_mml_field(#pname, params)?;
                        });
                        arm_bindings.push(quote! { #fname });
                        arm_steps.push(quote! {
                            <#fty as ::mml_def::MmlField>::append_mml_field(#fname, #pname, out)?;
                        });
                    }

                    build_parts.push(quote! { #fname: #fname });
                }

                parse_checks.push(quote! {
                    if tag_value.eq_ignore_ascii_case(#tag_value) {
                        #(#parse_steps)*
                        return Ok(Self::#vname {
                            #(#build_parts,)*
                        });
                    }
                });

                let pattern = if arm_bindings.is_empty() {
                    quote! { Self::#vname { .. } }
                } else {
                    quote! { Self::#vname { #(#arm_bindings,)* .. } }
                };

                serialize_arms.push(quote! {
                    #pattern => {
                        out.push((actual_tag_key.to_string(), #tag_value.to_string()));
                        #(#arm_steps)*
                        Ok(())
                    }
                });
            }
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new(
                    fields.span(),
                    "MmlBranch does not support tuple variants",
                ));
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::mml_def::MmlBranch for #name #ty_generics #where_clause {
            const TAG: Option<&'static str> = #tag_const;

            fn from_mml_branch(
                tag_key: &str,
                params: &::mml_def::MmlParams,
            ) -> Result<Self, ::mml_def::MmlError> {
                let actual_tag_key = Self::TAG.unwrap_or(tag_key);
                let raw_tag_value = params
                    .get(actual_tag_key)
                    .ok_or_else(|| ::mml_def::MmlError::MissingField(actual_tag_key.to_string()))?;
                let tag_value = ::mml_def::parse_plain_token(raw_tag_value)?;

                #(#parse_checks)*

                Err(::mml_def::MmlError::UnknownBranchTag {
                    tag: actual_tag_key.to_string(),
                    value: tag_value,
                })
            }

            fn append_mml_branch(
                &self,
                tag_key: &str,
                out: &mut Vec<(String, String)>,
            ) -> Result<(), ::mml_def::MmlError> {
                let actual_tag_key = Self::TAG.unwrap_or(tag_key);
                match self {
                    #(#serialize_arms,)*
                }
            }
        }

        impl #impl_generics ::mml_def::MmlField for #name #ty_generics #where_clause {
            fn has_field(field_name: &str, params: &::mml_def::MmlParams) -> bool {
                let tag_key = <Self as ::mml_def::MmlBranch>::TAG.unwrap_or(field_name);
                params.contains(tag_key)
            }

            fn from_mml_field(
                field_name: &str,
                params: &::mml_def::MmlParams,
            ) -> Result<Self, ::mml_def::MmlError> {
                let tag_key = <Self as ::mml_def::MmlBranch>::TAG.unwrap_or(field_name);
                <Self as ::mml_def::MmlBranch>::from_mml_branch(tag_key, params)
            }

            fn append_mml_field(
                &self,
                field_name: &str,
                out: &mut Vec<(String, String)>,
            ) -> Result<(), ::mml_def::MmlError> {
                let tag_key = <Self as ::mml_def::MmlBranch>::TAG.unwrap_or(field_name);
                <Self as ::mml_def::MmlBranch>::append_mml_branch(self, tag_key, out)
            }
        }
    })
}

#[proc_macro_derive(MmlValueEnum, attributes(mml))]
pub fn derive_mml_value_enum(input: TokenStream) -> TokenStream {
    derive_mml_value_enum_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive_mml_value_enum_impl(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let generics = input.generics;
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return Err(syn::Error::new(
                name.span(),
                "MmlValueEnum can only be derived for enum",
            ))
        }
    };

    let mut from_checks = Vec::new();
    let mut display_arms = Vec::new();

    for variant in variants {
        let cfg = parse_mml_variant_attrs(&variant)?;
        let vname = variant.ident;
        let lit = cfg.rename.unwrap_or_else(|| {
            let s = vname.to_string().to_ascii_uppercase();
            LitStr::new(&s, vname.span())
        });

        match variant.fields {
            Fields::Unit => {
                from_checks.push(quote! {
                    if s.eq_ignore_ascii_case(#lit) {
                        return Ok(Self::#vname);
                    }
                });
                display_arms.push(quote! {
                    Self::#vname => #lit,
                });
            }
            _ => {
                return Err(syn::Error::new(
                    variant.fields.span(),
                    "MmlValueEnum only supports unit variants",
                ));
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics std::str::FromStr for #name #ty_generics #where_clause {
            type Err = ::mml_def::InvalidEnumValue;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                #(#from_checks)*
                Err(::mml_def::InvalidEnumValue)
            }
        }

        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    #(#display_arms)*
                };
                write!(f, "{}", s)
            }
        }

        impl #impl_generics ::mml_def::MmlValue for #name #ty_generics #where_clause {
            fn from_mml_value(raw: &str) -> Result<Self, ::mml_def::MmlError> {
                let v = ::mml_def::parse_plain_token(raw)?;
                v.parse::<Self>().map_err(|_| {
                    ::mml_def::MmlError::InvalidParam(format!("invalid {}: {}", stringify!(#name), v))
                })
            }

            fn to_mml_value(&self) -> String {
                self.to_string()
            }
        }
    })
}
