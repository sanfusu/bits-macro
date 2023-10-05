#![allow(dead_code)]

use quote::{quote, ToTokens};
use syn::{parse::Parse, DeriveInput, Ident, Visibility};

use crate::parse::bits_field::BitsField;

use self::{bits_attr::BitStructAttr, bits_field::BitFieldPerm};

pub mod bits_attr;
pub mod bits_field;

pub struct BitStructItem {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Vec<bits_field::BitsField>,
    pub doc: Vec<syn::Attribute>,
}

impl Parse for BitStructItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input_ast = DeriveInput::parse(input)?;
        if input_ast.generics.lt_token.is_some() {
            return Err(syn::Error::new_spanned(
                input_ast.generics.into_token_stream(),
                "Generic is not allowed",
            ));
        }
        // extract doc here
        let doc = input_ast
            .attrs
            .clone()
            .into_iter()
            .filter(|x| x.path().is_ident("doc"))
            .collect();
        let raw_fields = match input_ast.data {
            syn::Data::Struct(x) => Ok(x.fields),
            _ => Err(syn::Error::new_spanned(
                input_ast.to_token_stream(),
                "Only struct is allowed",
            )),
        }?;
        if raw_fields.is_empty() {
            return Err(syn::Error::new_spanned(
                raw_fields.to_token_stream(),
                "Empty field is disallowed",
            ));
        }
        let mut fields = Vec::<bits_field::BitsField>::new();
        for field in raw_fields {
            let info = BitsField::try_from(field)?;
            fields.push(info);
        }
        Ok(BitStructItem {
            vis: input_ast.vis,
            name: input_ast.ident,
            fields,
            doc,
        })
    }
}

pub struct BitStruct {
    pub item: BitStructItem,
    pub attr: BitStructAttr,
}
impl BitStruct {
    pub fn new(attr: BitStructAttr, item: BitStructItem) -> BitStruct {
        BitStruct { attr, item }
    }
}
impl ToTokens for BitStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let c_name = &self.item.name;

        let base_ty = &self.attr.base_ty;
        let raw_vis = &self.attr.export;
        let vis = &self.item.vis;
        let doc = &self.item.doc;
        // 这里我们不更改 ident 的命名风格，否则会对 rust-analyzer 等 lint 工具产生误导。
        tokens.extend(quote! {
            #(#doc)*
            #vis struct #c_name(#raw_vis #base_ty);
            impl ::bits::Bitalized for #c_name {
                type BaseType = #base_ty;
            }
        });
        for field in &self.item.fields {
            let field_name = &field.name;
            let expr_range = &field.attr.expr;
            let target_ty = &field.target_ty;
            let doc = &field.doc;
            let vis = &field.vis;
            tokens.extend(quote! {
                #(#doc)*
                #vis struct #field_name;
                impl ::bits::Field for #field_name {
                    type CacheType = #target_ty;
                }
            });
            if (field.attr.perm == BitFieldPerm::R || field.attr.perm == BitFieldPerm::RW)
                && field.attr.need_try == false
            {
                tokens.extend(quote! {
                    impl ::bits::ReadField<#field_name> for #c_name {
                        fn read(&self, field: #field_name) -> #target_ty {
                            ::bits::Bits(self.0).read(#expr_range)
                        }
                    }
                });
            }
            if field.attr.perm == BitFieldPerm::W || field.attr.perm == BitFieldPerm::RW {
                tokens.extend(quote! {
                    impl ::bits::WriteField<#field_name> for #c_name {
                        fn write(&mut self, field: #field_name, v: #target_ty) {
                            ::bits::BitsMut(&mut self.0).write(#expr_range, v.into());
                        }
                    }
                });
            }
            if field.attr.need_try == true {
                tokens.extend(quote! {
                    impl ::bits::TryReadField<#field_name> for #c_name {
                        type Error = <#target_ty as TryFrom<Self::BaseType>>::Error;
                        fn try_read(&self, field: #field_name) -> Result<#target_ty, Self::Error> {
                            ::bits::Bits(self.0).read(#expr_range).try_into()
                        }
                    }
                });
            }
        }
    }
}
