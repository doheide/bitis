extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{Data, DeriveInput};
use regex::Regex;

// Generated code refers to the runtime as `::bitis_lib::...` (the library name of the `bitis`
// crate) and to std items as `::core::...`, so no glob import is required at the use site.

fn to_token_stream(r: syn::Result<proc_macro2::TokenStream>) -> TokenStream {
    match r {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Reads the mandatory `#[biserdi_enum_id_dynbits(N)]` attribute (1 <= N <= 32).
fn parse_enum_id_dynbits(input: &DeriveInput) -> syn::Result<u8> {
    const MSG: &str = "One instance of biserdi_enum_id_dynbits is required with one unsigned integer \
        between 1 and 32 as attribute, e.g. #[biserdi_enum_id_dynbits(4)].";
    let attrs: Vec<_> = input.attrs.iter()
        .filter(|attr| attr.path().is_ident("biserdi_enum_id_dynbits")).collect();
    if attrs.len() != 1 {
        return Err(syn::Error::new_spanned(&input.ident, MSG));
    }
    let lit: syn::LitInt = attrs[0].parse_args()
        .map_err(|e| syn::Error::new(e.span(), MSG))?;
    let v = lit.base10_parse::<u8>().map_err(|e| syn::Error::new(e.span(), MSG))?;
    if v == 0 || v > 32 {
        return Err(syn::Error::new_spanned(lit, MSG));
    }
    Ok(v)
}

//#[proc_macro_derive(BiserdiMsg, attributes(biserdi_enum))]
#[proc_macro_derive(BiserdiMsg)]
pub fn biserdi_msg(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    to_token_stream(biserdi_msg_impl(&input))
}
fn biserdi_msg_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_or_enum_identifier = &input.ident;
    let fields = match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => fields,
        _ => return Err(syn::Error::new_spanned(struct_or_enum_identifier, "BiserdiMsg is only allowed for structs")),
    };

    let mut bit_serialize_impl = quote!{};
    let mut bit_deserialize_impl = quote!{};
    let mut bit_deserialize_impl_init = quote!{};
    let mut bit_deserialize_impl_size = quote!{0_u64};
    let mut msg_display_impl = quote!{};
    let mut bit_min_size_impl = quote!{};
    let mut bit_max_size_impl = quote!{};

    for field in fields.iter() {
        let identifier = match field.ident.as_ref() {
            Some(i) => i,
            None => return Err(syn::Error::new_spanned(field, "BiserdiMsg requires named fields")),
        };
        let ty = &field.ty;
        let temp_identifier = format_ident!("t_{}", identifier);

        bit_serialize_impl.extend(quote!{
            s += ::bitis_lib::BiserdiTrait::bit_serialize(&self.#identifier, biseri)?;
        });
        bit_deserialize_impl.extend(quote!{
            let #temp_identifier = <#ty as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
        });
        bit_deserialize_impl_init.extend(quote!{
            #identifier: #temp_identifier.0,
        });
        bit_deserialize_impl_size.extend(quote!{
            +#temp_identifier.1
        });
        msg_display_impl.extend(quote!{
            ::core::write!(f, "{}: {}, ", ::core::stringify!(#identifier), self.#identifier)?;
        });
        bit_min_size_impl.extend(quote! {
            bs = bs.saturating_add(<#ty as ::bitis_lib::BiserdiTrait>::min_bits());
        });
        bit_max_size_impl.extend(quote! {
            bs = bs.checked_add(<#ty as ::bitis_lib::BiserdiTrait>::max_bits()?)?;
        });
    }

    Ok(quote! {
        #[automatically_derived]
        impl ::bitis_lib::BiserdiTrait for #struct_or_enum_identifier {
            fn bit_serialize(self: &Self, biseri: &mut ::bitis_lib::Biseri) -> ::core::option::Option<u64> {
                let mut s = 0_u64;
                #bit_serialize_impl
                ::core::option::Option::Some(s)
            }
            fn bit_deserialize(version_id: u16, bides: &mut ::bitis_lib::Bides) -> ::core::option::Option<(Self, u64)> {
                let _ = &version_id;
                let _ = &bides;
                #bit_deserialize_impl
                ::core::option::Option::Some((Self{#bit_deserialize_impl_init}, #bit_deserialize_impl_size))
            }
            fn min_bits() -> u64 {
                let mut bs: u64 = 0;
                #bit_min_size_impl
                bs
            }
            fn max_bits() -> ::core::option::Option<u64> {
                let mut bs: u64 = 0;
                #bit_max_size_impl
                ::core::option::Option::Some(bs)
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Display for #struct_or_enum_identifier {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::write!(f, "{}{{", ::core::stringify!(#struct_or_enum_identifier))?;
                #msg_display_impl
                ::core::write!(f, "}}")
            }
        }
    })
}

#[proc_macro_derive(BiserdiEnum, attributes(biserdi_enum_id_dynbits))]
pub fn biserdi_enum(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    to_token_stream(biserdi_enum_impl(&input))
}
fn biserdi_enum_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_or_enum_identifier = &input.ident;
    let variants = match &input.data {
        Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => return Err(syn::Error::new_spanned(struct_or_enum_identifier, "BiserdiEnum is only allowed for enums")),
    };
    let dyn_bits = parse_enum_id_dynbits(input)?;

    let mut bit_serialize_impl = quote!{};
    let mut bit_deserialize_impl = quote!{};

    for (id, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new_spanned(variant, "BiserdiEnum is only allowed for enums without nested types"));
        }
        let id_u32 = id as u32;

        bit_serialize_impl.extend(quote! {
            #struct_or_enum_identifier::#ident => #id_u32,
        });
        bit_deserialize_impl.extend(quote! {
            #id_u32 => #struct_or_enum_identifier::#ident,
        });
    }
    Ok(quote! {
        #[automatically_derived]
        // BiserdiEnum
        impl ::bitis_lib::BiserdiTrait for #struct_or_enum_identifier {
            fn bit_serialize(self: &Self, biseri: &mut ::bitis_lib::Biseri) -> ::core::option::Option<u64> {
                let id: u32 = match *self {
                    #bit_serialize_impl
                };
                ::bitis_lib::BiserdiTrait::bit_serialize(
                    &::bitis_lib::DynInteger::<u32, 32, #dyn_bits>::new(id), biseri)
            }
            #[allow(unreachable_code)]
            fn bit_deserialize(version_id: u16, bides: &mut ::bitis_lib::Bides) -> ::core::option::Option<(Self, u64)> {
                let oo_val = <::bitis_lib::DynInteger::<u32, 32, #dyn_bits> as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                ::core::option::Option::Some((match oo_val.0.val {
                    #bit_deserialize_impl
                    _ => { return bides.fail(::bitis_lib::DecodeError::InvalidDiscriminant) }
                }, oo_val.1))
            }
            fn min_bits() -> u64 { 1 }
        }
        #[automatically_derived]
        impl ::core::fmt::Display for #struct_or_enum_identifier {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::write!(f, "{:?}", self)
            }
        }
    })
}

#[proc_macro_derive(BiserdiOneOf, attributes(biserdi_enum_id_dynbits))]
pub fn biserdi_one_of(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    to_token_stream(biserdi_one_of_impl(&input))
}
fn biserdi_one_of_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_or_enum_identifier = &input.ident;
    let variants = match &input.data {
        Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => return Err(syn::Error::new_spanned(struct_or_enum_identifier, "BiserdiOneOf is only allowed for enums")),
    };
    let dyn_bits = parse_enum_id_dynbits(input)?;

    let mut bit_serialize_impl = quote!{};
    let mut bit_deserialize_impl = quote!{};
    let mut bit_min_size_impl = quote!{};

    for (id, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;
        let ty = match &variant.fields {
            syn::Fields::Unnamed(ty) if ty.unnamed.len() == 1 => &ty.unnamed[0].ty,
            _ => return Err(syn::Error::new_spanned(variant,
                "BiserdiOneOf is only allowed for enums whose variants have exactly one unnamed field")),
        };
        let id_u32 = id as u32;

        bit_serialize_impl.extend(quote! {
            #struct_or_enum_identifier::#ident(v) => {
                let s = ::bitis_lib::BiserdiTrait::bit_serialize(
                    &::bitis_lib::DynInteger::<u32, 32, #dyn_bits>::new(#id_u32), biseri)?;
                s + ::bitis_lib::BiserdiTrait::bit_serialize(v, biseri)?
            },
        });
        bit_deserialize_impl.extend(quote! {
            #id_u32 => {
                let v = <#ty as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                (#struct_or_enum_identifier::#ident(v.0), v.1 + oo_val.1)
            },
        });
        bit_min_size_impl.extend(quote! {
            let m = <#ty as ::bitis_lib::BiserdiTrait>::min_bits();
            bs = ::core::option::Option::Some(match bs { ::core::option::Option::Some(b) => ::core::cmp::min(b, m), ::core::option::Option::None => m });
        });
    }
    let serialize_match = if variants.is_empty() {
        quote! { match *self {} }
    } else {
        quote! { ::core::option::Option::Some(match self { #bit_serialize_impl }) }
    };
    Ok(quote! {
        #[automatically_derived]
        // BiserdiOneOf
        impl ::bitis_lib::BiserdiTrait for #struct_or_enum_identifier {
            fn bit_serialize(self: &Self, biseri: &mut ::bitis_lib::Biseri) -> ::core::option::Option<u64> {
                let _ = &biseri;
                #serialize_match
            }
            #[allow(unreachable_code)]
            fn bit_deserialize(version_id: u16, bides: &mut ::bitis_lib::Bides) -> ::core::option::Option<(Self, u64)> {
                let oo_val = <::bitis_lib::DynInteger::<u32, 32, #dyn_bits> as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                ::core::option::Option::Some(match oo_val.0.val {
                    #bit_deserialize_impl
                    _ => { return bides.fail(::bitis_lib::DecodeError::InvalidDiscriminant) }
                })
            }
            /// 1 bit discriminant + minimum over the variants (spec section 4).
            fn min_bits() -> u64 {
                #[allow(unused_mut)]
                let mut bs: ::core::option::Option<u64> = ::core::option::Option::None;
                #bit_min_size_impl
                1_u64.saturating_add(bs.unwrap_or(0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Display for #struct_or_enum_identifier {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::write!(f, "{:?}", self)
            }
        }
    })
}


#[proc_macro_derive(BiserdiMsgVersioned)]
pub fn biserdi_msg_versioned(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    to_token_stream(biserdi_msg_versioned_impl(&input))
}
fn biserdi_msg_versioned_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_or_enum_identifier = &input.ident;
    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            const MSG: &str = "BiserdiMsgVersioned has to have a field 'base' and a field 'ext' and no other.";
            let mut base_ty = None;
            let mut ext_ty = None;
            for field in fields {
                match field.ident.as_ref().map(|i| i.to_string()).as_deref() {
                    Some("base") => { base_ty = Some(field.ty.clone()); }
                    Some("ext") => { ext_ty = Some(field.ty.clone()); }
                    _ => return Err(syn::Error::new_spanned(field, MSG)),
                }
            }
            let (base_ty, ext_ty) = match (base_ty, ext_ty) {
                (Some(b), Some(e)) => (b, e),
                _ => return Err(syn::Error::new_spanned(struct_or_enum_identifier, MSG)),
            };
            Ok(quote! {
                #[automatically_derived]
                impl ::bitis_lib::BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut ::bitis_lib::Biseri) -> ::core::option::Option<u64> {
                        // send base in any case
                        let mut total_size = ::bitis_lib::BiserdiTrait::bit_serialize(&self.base, biseri)?;

                        // send ext with size
                        let mut biseri_temp = ::bitis_lib::Biseri::new();
                        let dyn_msg_size = ::bitis_lib::BiserdiTrait::bit_serialize(&self.ext, &mut biseri_temp)?;
                        biseri_temp.finish_add_data();

                        total_size += ::bitis_lib::BiserdiTrait::bit_serialize(
                            &::bitis_lib::DynInteger::<u64, 64, 4>::new(dyn_msg_size), biseri)?;
                        // also propagates encoding errors of ext
                        total_size += biseri.add_biseri_data(&biseri_temp)?;

                        ::core::option::Option::Some(total_size)
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut ::bitis_lib::Bides) -> ::core::option::Option<(Self, u64)> {
                        let (base, base_size) = <#base_ty as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;

                        let (ext_size, ext_size_bits) = <::bitis_lib::DynInteger<u64, 64, 4> as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                        let ext_size = ext_size.val;
                        if ext_size > bides.remaining_bits() {
                            return bides.fail(::bitis_lib::DecodeError::UnexpectedEnd);
                        }
                        let (ext, cur_ext_size) = <#ext_ty as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                        if cur_ext_size > ext_size {
                            return bides.fail(::bitis_lib::DecodeError::InvalidValue);
                        }
                        bides.try_skip_bits(ext_size - cur_ext_size)?;

                        let total_size = match base_size.checked_add(ext_size_bits).and_then(|s| s.checked_add(ext_size)) {
                            ::core::option::Option::Some(s) => s,
                            ::core::option::Option::None => { return bides.fail(::bitis_lib::DecodeError::InvalidValue); }
                        };
                        ::core::option::Option::Some((Self{base, ext}, total_size))
                    }
                    fn min_bits() -> u64 {
                        // base + at least one bit for the size of ext
                        <#base_ty as ::bitis_lib::BiserdiTrait>::min_bits().saturating_add(1)
                    }
                }
            })
        },
        Data::Enum(syn::DataEnum { variants, .. }) => {
            let re = Regex::new(r"V([0-9]+)").unwrap();
            let mut bit_serialize_impl = quote! {};
            let mut bit_deserialize_impl = quote! {};

            for variant in variants.iter() {
                let ident = &variant.ident;
                let ver_num = match re.captures(&ident.to_string())
                    .and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<u16>().ok()) {
                    Some(v) => v,
                    None => return Err(syn::Error::new_spanned(ident,
                        "VersionEnums for Biserdi need to have variants in the form of V[0-9]+ (version number fitting into u16)")),
                };

                let ty = match &variant.fields {
                    syn::Fields::Unnamed(ty) if ty.unnamed.len() == 1 => &ty.unnamed[0].ty,
                    _ => return Err(syn::Error::new_spanned(variant,
                        "BiserdiMsgVersioned enums are only allowed with exactly one unnamed field per variant")),
                };

                bit_serialize_impl.extend(quote! {
                    #struct_or_enum_identifier::#ident(v) => ::bitis_lib::BiserdiTrait::bit_serialize(v, biseri)?, });
                bit_deserialize_impl.extend(quote! {
                    #ver_num => {
                        let v = <#ty as ::bitis_lib::BiserdiTrait>::bit_deserialize(version_id, bides)?;
                        (#struct_or_enum_identifier::#ident(v.0), v.1)
                    },
                });
            }
            let serialize_match = if variants.is_empty() {
                quote! { match *self {} }
            } else {
                quote! { ::core::option::Option::Some(match self { #bit_serialize_impl }) }
            };
            Ok(quote! {
                #[automatically_derived]
                // BiserdiMsgVersioned
                impl ::bitis_lib::BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut ::bitis_lib::Biseri) -> ::core::option::Option<u64> {
                        let _ = &biseri;
                        #serialize_match
                    }
                    #[allow(unreachable_code)]
                    fn bit_deserialize(version_id: u16, bides: &mut ::bitis_lib::Bides) -> ::core::option::Option<(Self, u64)> {
                        ::core::option::Option::Some(match version_id {
                            #bit_deserialize_impl
                            // unknown version requested by the caller
                            _ => { return bides.fail(::bitis_lib::DecodeError::InvalidValue) }
                        })
                    }
                    fn min_bits() -> u64 { 0 }
                }
            })
        },
        _ => Err(syn::Error::new_spanned(struct_or_enum_identifier, "BiserdiMsgVersioned is only allowed for structs and enums")),
    }
}
