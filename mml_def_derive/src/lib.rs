use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Field, Fields,
    GenericArgument, GenericParam, Generics, Lifetime, LitStr, Result, Type,
    TypePath, Variant,
};

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

fn with_de_generics(generics: &Generics) -> Generics {
    let mut out = generics.clone();
    out.params
        .insert(0, GenericParam::Lifetime(parse_quote!('de)));

    let lifetime_params: Vec<Lifetime> = out
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Lifetime(lp) if lp.lifetime.ident != "de" => Some(lp.lifetime.clone()),
            _ => None,
        })
        .collect();

    if !lifetime_params.is_empty() {
        let where_clause = out.make_where_clause();
        for lt in lifetime_params {
            where_clause.predicates.push(parse_quote!('de: #lt));
        }
    }

    out
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
                let #ident = if <#inner_ty as ::mml_def::__private::MmlField<'de>>::has_field(#pname, params) {
                    Some(<#inner_ty as ::mml_def::__private::MmlField<'de>>::from_mml_field(#pname, params)?)
                } else {
                    None
                };
            });
            ser_steps.push(quote! {
                if let Some(value) = &self.#ident {
                    <#inner_ty as ::mml_def::__private::MmlField<'_>>::append_mml_field(value, #pname, &mut out)?;
                }
            });
        } else {
            let ty = field.ty;
            deser_steps.push(quote! {
                let #ident = <#ty as ::mml_def::__private::MmlField<'de>>::from_mml_field(#pname, params)?;
            });
            ser_steps.push(quote! {
                <#ty as ::mml_def::__private::MmlField<'_>>::append_mml_field(&self.#ident, #pname, &mut out)?;
            });
        }

        build_fields.push(quote! { #ident });
    }

    let de_generics = with_de_generics(&generics);
    let (de_impl_generics, _, de_where_clause) = de_generics.split_for_impl();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #de_impl_generics ::mml_def::__private::MmlDeserialize<'de> for #name #ty_generics #de_where_clause {
            fn from_mml_params<P>(params: P) -> Result<Self, ::mml_def::MmlError>
            where
                P: ::mml_def::__private::MmlParamLookup<'de> + Copy,
            {
                #(#deser_steps)*
                Ok(Self {
                    #(#build_fields,)*
                })
            }
        }

        impl #impl_generics ::mml_def::__private::MmlSerialize for #name #ty_generics #where_clause {
            fn to_mml_params(&self) -> Result<Vec<(String, String)>, ::mml_def::MmlError> {
                let mut out = Vec::new();
                #(#ser_steps)*
                Ok(out)
            }
        }

        impl #impl_generics ::mml_def::__private::MmlCommand for #name #ty_generics #where_clause {
            const MML_OP: &'static str = #op;
            const MML_OBJECT: &'static str = #object;
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn from_mml_line(input: &str) -> Result<Self, ::mml_def::MmlError>
            where
                for<'de2> Self: ::mml_def::__private::MmlDeserialize<'de2>,
            {
                <Self as ::mml_def::__private::MmlCommand>::from_mml_line(input)
            }

            pub fn from_mml_line_borrowed<'de2>(
                input: &'de2 str,
            ) -> Result<Self, ::mml_def::MmlError>
            where
                Self: ::mml_def::__private::MmlDeserialize<'de2>,
            {
                <Self as ::mml_def::__private::MmlCommand>::from_mml_line_borrowed(input)
            }

            pub fn from_mml_line_ref<'de2>(
                input: &'de2 str,
            ) -> Result<Self, ::mml_def::MmlError>
            where
                Self: ::mml_def::__private::MmlDeserialize<'de2>,
            {
                Self::from_mml_line_borrowed(input)
            }

            pub fn to_mml_line(&self) -> Result<String, ::mml_def::MmlError> {
                <Self as ::mml_def::__private::MmlCommand>::to_mml_line(self)
            }

            pub const fn mml_op() -> &'static str {
                <Self as ::mml_def::__private::MmlCommand>::MML_OP
            }

            pub const fn mml_object() -> &'static str {
                <Self as ::mml_def::__private::MmlCommand>::MML_OBJECT
            }
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
                            let #fname = if <#inner_ty as ::mml_def::__private::MmlField<'de>>::has_field(#pname, params) {
                                Some(<#inner_ty as ::mml_def::__private::MmlField<'de>>::from_mml_field(#pname, params)?)
                            } else {
                                None
                            };
                        });
                        arm_bindings.push(quote! { #fname });
                        arm_steps.push(quote! {
                            if let Some(value) = #fname {
                                <#inner_ty as ::mml_def::__private::MmlField<'_>>::append_mml_field(value, #pname, out)?;
                            }
                        });
                    } else {
                        let fty = field.ty;
                        parse_steps.push(quote! {
                            let #fname = <#fty as ::mml_def::__private::MmlField<'de>>::from_mml_field(#pname, params)?;
                        });
                        arm_bindings.push(quote! { #fname });
                        arm_steps.push(quote! {
                            <#fty as ::mml_def::__private::MmlField<'_>>::append_mml_field(#fname, #pname, out)?;
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

    let de_generics = with_de_generics(&generics);
    let (de_impl_generics, _, de_where_clause) = de_generics.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();

    Ok(quote! {
        impl #de_impl_generics ::mml_def::__private::MmlBranch<'de> for #name #ty_generics #de_where_clause {
            const TAG: Option<&'static str> = #tag_const;

            fn from_mml_branch<P>(
                tag_key: &str,
                params: P,
            ) -> Result<Self, ::mml_def::MmlError>
            where
                P: ::mml_def::__private::MmlParamLookup<'de> + Copy,
            {
                let actual_tag_key = Self::TAG.unwrap_or(tag_key);
                let raw_tag_value = params
                    .get(actual_tag_key)
                    .ok_or_else(|| ::mml_def::MmlError::MissingField(actual_tag_key.to_string()))?;
                let tag_value = ::mml_def::__private::parse_plain_token_ref(raw_tag_value)?;

                #(#parse_checks)*

                Err(::mml_def::MmlError::UnknownBranchTag {
                    tag: actual_tag_key.to_string(),
                    value: tag_value.to_string(),
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

        impl #de_impl_generics ::mml_def::__private::MmlField<'de> for #name #ty_generics #de_where_clause {
            fn has_field<P>(field_name: &str, params: P) -> bool
            where
                P: ::mml_def::__private::MmlParamLookup<'de> + Copy,
            {
                let tag_key = <Self as ::mml_def::__private::MmlBranch<'de>>::TAG.unwrap_or(field_name);
                params.contains(tag_key)
            }

            fn from_mml_field<P>(
                field_name: &str,
                params: P,
            ) -> Result<Self, ::mml_def::MmlError>
            where
                P: ::mml_def::__private::MmlParamLookup<'de> + Copy,
            {
                let tag_key = <Self as ::mml_def::__private::MmlBranch<'de>>::TAG.unwrap_or(field_name);
                <Self as ::mml_def::__private::MmlBranch<'de>>::from_mml_branch(tag_key, params)
            }

            fn append_mml_field(
                &self,
                field_name: &str,
                out: &mut Vec<(String, String)>,
            ) -> Result<(), ::mml_def::MmlError> {
                let tag_key = <Self as ::mml_def::__private::MmlBranch<'de>>::TAG.unwrap_or(field_name);
                <Self as ::mml_def::__private::MmlBranch<'de>>::append_mml_branch(self, tag_key, out)
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
    let mut expected_values = Vec::new();

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
                expected_values.push(lit);
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
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                #(#from_checks)*
                Err(format!("invalid {}: {}", stringify!(#name), s))
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
                let v = ::mml_def::__private::parse_plain_token(raw)?;
                v.parse::<Self>().map_err(|e| {
                    let expected = [#(#expected_values),*].join(", ");
                    ::mml_def::MmlError::InvalidParam(format!(
                        "{} (expected one of: {})",
                        e,
                        expected
                    ))
                })
            }

            fn to_mml_value(&self) -> String {
                self.to_string()
            }
        }
    })
}

















