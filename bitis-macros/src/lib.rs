extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::{quote};
use syn::Data;
use proc_macro2::{Span, Ident};
use proc_macro2::TokenTree::Literal;
use regex::Regex;

//#[proc_macro_derive(BiserdiMsg, attributes(biserdi_enum))]
#[proc_macro_derive(BiserdiMsg)]
pub fn biserdi_msg(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_or_enum_identifier = &input.ident;
    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let mut bit_serialize_impl = quote!{};
            let mut bit_deserialize_impl = quote!{};
            let mut bit_deserialize_impl_init = quote!{};
            let mut bit_deserialize_impl_size = quote!{0};
            let mut msg_display_impl = quote!{};

            let size_identifier = Ident::new("s".into(), Span::call_site());
            // let size_identifier = quote::format_ident!("s");
            let bit_serialize_self_identifier = quote::format_ident!("self");

            for (_idx, field) in fields.iter().enumerate() {
                let identifier = field.ident.as_ref().unwrap();
                let ty = field.ty.clone();
                let temp_identifier = quote::format_ident!("t_{}", identifier);

                bit_serialize_impl.extend(quote!{
                    #size_identifier += #bit_serialize_self_identifier.#identifier.bit_serialize(biseri)?;
                });
                bit_deserialize_impl.extend(quote!{
                    let #temp_identifier = call_deserialize::<#ty>(version_id, bides)?;
                });
                bit_deserialize_impl_init.extend(quote!{
                    #identifier: #temp_identifier.0,
                });
                bit_deserialize_impl_size.extend(quote!{
                    +#temp_identifier.1
                });
                msg_display_impl.extend(quote!{
                    write!(f, "{}: {}, ", stringify!(#identifier), #bit_serialize_self_identifier.#identifier)?;
                });
            }

            let code = quote! {
                #[automatically_derived]
                impl BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                        let mut #size_identifier = 0_u64;
                        #bit_serialize_impl
                        Some(#size_identifier)
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                        fn call_deserialize<T:BiserdiTrait>(version_id: u16, bides: &mut Bides) -> Option<(T, u64)> {
                            T::bit_deserialize(version_id, bides) }
                        #bit_deserialize_impl
                        Some((Self{#bit_deserialize_impl_init}, #bit_deserialize_impl_size))
                    }
                }
                #[automatically_derived]
                impl std::fmt::Display for #struct_or_enum_identifier {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}{{", stringify!(#struct_or_enum_identifier))?;
                        #msg_display_impl
                        write!(f, "}}")
                    }
                }
            };
            // println!("{}", code);
            code
        },
        _ => panic!("BiserdiMsg only allowed for Structs")
    }.into()
}

#[proc_macro_derive(BiserdiEnum, attributes(biserdi_enum_id_dynbits))]
pub fn biserdi_enum(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_or_enum_identifier = &input.ident;
    // println!("input: {:?}", input);

    if input.attrs.len() == 0 {
        panic!("One instance of biserdi_enum_id_dynbits is required with one unsigned integer as attribute (8-bit), e.g. #[biserdi_enum_id_dynbits(4)].")
    }
    // println!("meta: {:?}", input.attrs[0].meta);


    let v: Vec<_> = input.attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("biserdi_enum_id_dynbits") {
            let v = attr.meta.require_list().clone().unwrap().tokens.clone().into_iter()
                .filter_map(|x| {
                match x { Literal(v) => Some(v.to_string().parse::<u8>().ok()), _ => None }
            }).collect::<Vec<_>>();
            Some(v)
        } else { None }
    }).collect::<Vec<_>>().concat();
    if v.len() != 1 {
        panic!("One instance of biserdi_enum_id_dynbits is required with one unsigned integer as attribute (8-bit), e.g. #[biserdi_enum_id_dynbits(4)].")
    }
    let dyn_bits = v[0];

    match &input.data {
        Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut bit_serialize_impl = quote!{};
            let mut bit_deserialize_impl = quote!{};

            for (id, variant) in variants.iter().enumerate() {
                let ident = variant.ident.clone();
                match variant.fields.clone() {
                    syn::Fields::Named(_) => panic!("Biserdi for enum only allowed witout nested types"),
                    syn::Fields::Unnamed(_) => panic!("Biserdi for enum only allowed witout nested types"),
                    syn::Fields::Unit => (),
                };
                // println!("ty: {:?}", ty);
                let id_u32 = id as u32;
                let id_token = quote! { #id_u32 };

                bit_serialize_impl.extend(quote! {
                    #struct_or_enum_identifier::#ident => {
                        DynInteger::<u32, 32, #dyn_bits>::new(#id_token).bit_serialize(biseri)?
                    },
                });
                bit_deserialize_impl.extend(quote! {
                    #id_token => {
                        #struct_or_enum_identifier::#ident
                    },
                });
            }
            let code = quote! {
                #[automatically_derived]
                // BiserdiOneOf
                impl BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                        Some(match self {
                            #bit_serialize_impl
                        })
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                        fn call_deserialize<T:BiserdiTrait>(version_id: u16, bides: &mut Bides) -> Option<(T, u64)> {
                            T::bit_deserialize(version_id, bides) }
                        let oo_val = DynInteger::<u32, 32, #dyn_bits>::bit_deserialize(version_id, bides)?;
                        Some((match oo_val.0.val {
                            #bit_deserialize_impl
                            _ => { return None }
                        }, oo_val.1))
                    }
                }
            };
            // println!("{}", code);
            code
        },
        _ => panic!("BiserdiEnum only allowed for Enums")
    }.into()
}

#[proc_macro_derive(BiserdiOneOf, attributes(biserdi_enum_id_dynbits))]
pub fn biserdi_one_of(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_or_enum_identifier = &input.ident;
    // println!("input: {:?}", input);

    let v: Vec<_> = input.attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("biserdi_enum_id_dynbits") {
            let v = attr.meta.require_list().clone().unwrap().tokens.clone().into_iter()
                .filter_map(|x| {
                    match x { Literal(v) => Some(v.to_string().parse::<u8>().ok()), _ => None }
                }).collect::<Vec<_>>();
            Some(v)
        } else { None }
    }).collect::<Vec<_>>().concat();
    if v.len() != 1 {
        panic!("One instance of biserdi_enum_id_dynbits is required with one unsigned integer as attribute (8-bit), e.g. #[biserdi_enum_id_dynbits(4)].")
    }
    let dyn_bits = v[0];

    match &input.data {
        Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut bit_serialize_impl = quote!{};
            let mut bit_deserialize_impl = quote!{};

            for (id, variant) in variants.iter().enumerate() {
                let ident = variant.ident.clone();
                let ty = match variant.fields.clone() {
                    syn::Fields::Named(_) => panic!("Biserdi for enum only allowed with unnamed fields"),
                    syn::Fields::Unnamed(ty) => ty.unnamed.clone(),
                    syn::Fields::Unit => panic!("Biserdi for enum only allowed with unnamed fields"),
                };
                // println!("ty: {:?}", ty);
                let id_u32 = id as u32;
                let id_token = quote! { #id_u32 };

                bit_serialize_impl.extend(quote! {
                    #struct_or_enum_identifier::#ident(v) => {
                        let s = DynInteger::<u32, 32, #dyn_bits>::new(#id_token).bit_serialize(biseri)?;
                        s + v.bit_serialize(biseri)?
                    },
                });
                bit_deserialize_impl.extend(quote! {
                    #id_token => {
                        let v = call_deserialize::<#ty>(version_id, bides)?;
                        (#struct_or_enum_identifier::#ident(v.0), v.1 + oo_val.1)
                    },
                });
            }
            let code = quote! {
                #[automatically_derived]
                // BiserdiOneOf
                impl BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                        Some(match self {
                            #bit_serialize_impl
                        })
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                        fn call_deserialize<T:BiserdiTrait>(version_id: u16, bides: &mut Bides) -> Option<(T, u64)> {
                            T::bit_deserialize(version_id, bides) }
                        let oo_val = DynInteger::<u32, 32, #dyn_bits>::bit_deserialize(version_id, bides)?;
                        Some(match oo_val.0.val {
                            #bit_deserialize_impl
                            _ => { return None }
                        })
                    }
                }
            };
            // println!("{}", code);
            code
        },
        _ => panic!("BiserdiOneOf only allowed for Enums")
    }.into()
}


#[proc_macro_derive(BiserdiMsgVersioned)]
pub fn biserdi_msg_versioned(item: TokenStream) -> TokenStream {
    let re = Regex::new(r"V([0-9]+)").unwrap();

    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_or_enum_identifier = &input.ident;
    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let mut base_ty = None;
            let mut ext_ty = None;
            for field in fields {
                if field.ident.as_ref().unwrap().to_string() == String::from("base") {
                    base_ty = Some(field.ty.clone());
                }
                else if field.ident.as_ref().unwrap().to_string() == String::from("ext") {
                    ext_ty = Some(field.ty.clone());
                }
                else {
                    panic!("BiserdiMsgVersioned has to have a field 'base' and a field 'ext' and no other.")
                }
            }
            if base_ty.is_none() || ext_ty.is_none() {
                panic!("BiserdiMsgVersioned has to have a field 'base' and a field 'ext'.")
            }
            let code = quote! {
                #[automatically_derived]
                impl BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                        // send base in any case
                        let mut total_size = self.base.bit_serialize(biseri)?;

                        // send ext with size
                        let mut biseri_temp = Biseri::new();
                        let dyn_msg_size = self.ext.bit_serialize(&mut biseri_temp)?;
                        biseri_temp.finish_add_data();

                        total_size += DynInteger::<u64, 64, 4>::new(dyn_msg_size).bit_serialize(biseri)?;
                        total_size += biseri.add_biseri_data(&biseri_temp)?;

                        Some(total_size)
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                        fn call_deserialize<T: BiserdiTrait>(version_id: u16, bides: &mut Bides) -> Option<(T, u64)> {
                            T::bit_deserialize(version_id, bides)
                        }
                        let mut total_size = 0;

                        let (base, cur_size) = call_deserialize::<#base_ty>(version_id, bides)?;
                        total_size += cur_size;

                        let (ext_size, cur_size) = call_deserialize::<DynInteger<u64, 64, 4>>(version_id, bides)?;
                        total_size += ext_size.val + cur_size;
                        let (ext, cur_ext_size) = call_deserialize::<#ext_ty>(version_id, bides)?;
                        if cur_ext_size > ext_size.val { return None; }

                        let skip_bits = ext_size.val - cur_ext_size;
                        bides.skip_bits(skip_bits);

                        Some((Self{base, ext}, total_size))
                    }
                }
            };
            // println!("{}", code);
            code
        },
        Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut bit_serialize_impl = quote! {};
            let mut bit_deserialize_impl = quote! {};

            for variant in variants.iter() {
                let ident = variant.ident.clone();
                if !re.is_match(&ident.to_string()) { panic!("VersionEnums for Biserdi need to have variants in the form of V[0-9]+") }

                let ty = match variant.fields.clone() {
                    syn::Fields::Named(_) => panic!("Biserdi for enum only allowed with named fields"),
                    syn::Fields::Unnamed(ty) => ty.unnamed.clone(),
                    syn::Fields::Unit => panic!("Biserdi for enum only allowed for field with a type"),
                };

                bit_serialize_impl.extend(quote! {
                    #struct_or_enum_identifier::#ident(v) => v.bit_serialize(biseri)?, });
                fn get_capture_num(re: &Regex, str: &String) -> Option<u16>{
                    re.captures(str)?.get(1)?.as_str().parse::<u16>().ok()
                }
                let ver_num = get_capture_num(&re, &ident.to_string()).unwrap();
                bit_deserialize_impl.extend(quote! {
                    #ver_num => {
                        let v = call_deserialize::<#ty>(version_id, bides)?;
                        (#struct_or_enum_identifier::#ident(v.0), v.1)
                    },
                });
            }
            let code = quote! {
                #[automatically_derived]
                // BiserdiMsgVersioned
                impl BiserdiTrait for #struct_or_enum_identifier {
                    fn bit_serialize(self: &Self, biseri: &mut Biseri) -> Option<u64> {
                        Some(match self {
                            #bit_serialize_impl
                        })
                    }
                    fn bit_deserialize(version_id: u16, bides: &mut Bides) -> Option<(Self, u64)> {
                        fn call_deserialize<T:BiserdiTrait>(version_id: u16, bides: &mut Bides) -> Option<(T, u64)> {
                            T::bit_deserialize(version_id, bides) }
                        Some(match version_id.clone() {
                            #bit_deserialize_impl
                            _ => { return None }
                        })
                    }
                }
            };
            // println!("{}", code);
            code
        },
        _ => panic!("BiserdiMsgVersioned only allowed for Structs")
    }.into()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
