#![allow(dead_code)]

use quote::{quote, ToTokens};
use syn::{parse::Parse, DeriveInput, Ident, Visibility};

use crate::parse::field_info::BitStructFieldInfo;

use self::{container_attr::BitContainerAttr, field_info::BitFieldPerm};

pub mod container_attr;
pub mod field_info;

pub struct BitStructInfo {
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Vec<field_info::BitStructFieldInfo>,
    pub doc: Vec<syn::Attribute>,
}

impl Parse for BitStructInfo {
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
        let mut fields = Vec::<field_info::BitStructFieldInfo>::new();
        for field in raw_fields {
            let info = BitStructFieldInfo::try_from(field)?;
            fields.push(info);
        }
        Ok(BitStructInfo {
            vis: input_ast.vis,
            ident: input_ast.ident,
            fields,
            doc,
        })
    }
}

pub struct InnerField {
    pub perm: BitFieldPerm,
    pub pos: syn::ExprRange,
    pub need_try: bool,
    pub target_ty: syn::Type,
    pub ident: syn::Ident,
    pub vis: Visibility,
    pub doc: Vec<syn::Attribute>,
}
pub struct BitField {
    pub base_ty: syn::Path,
    pub ident: syn::Ident,
    pub vis: Visibility,
    pub export: Visibility,
    pub doc: Vec<syn::Attribute>,
    pub inner_fields: Vec<InnerField>,
}
impl BitField {
    pub fn new(c_info: BitContainerAttr, b_info: BitStructInfo) -> BitField {
        let mut inner_fields: Vec<InnerField> = Vec::new();
        for info in b_info.fields {
            let field = InnerField {
                perm: info.attr.perm,
                pos: info.attr.expr_range,
                need_try: info.attr.need_try,
                target_ty: info.target_ty,
                ident: info.name,
                vis: info.vis,
                doc: info.doc,
            };
            inner_fields.push(field);
        }
        BitField {
            base_ty: c_info.base_ty,
            ident: b_info.ident,
            vis: b_info.vis,
            export: c_info.export,
            doc: b_info.doc,
            inner_fields,
        }
    }
}
impl ToTokens for BitField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut field_scop = quote! {};
        let c_name = &self.ident;

        for field in &self.inner_fields {
            let field_name = &field.ident;
            let expr_range = &field.pos;
            let target_ty = &field.target_ty;
            let doc = &field.doc;
            let vis = &field.vis;
            let mut current = quote! {
                #(#doc)*
                #vis struct #field_name;
                impl ::bits::Field for #field_name {
                    type CacheType = #target_ty;
                }
            };
            if (field.perm == BitFieldPerm::R || field.perm == BitFieldPerm::RW)
                && field.need_try == false
            {
                current.extend(quote! {
                    impl ::bits::ReadField<#field_name> for #c_name {
                        fn read(&self, field: #field_name) -> #target_ty {
                            ::bits::Bits(self.0).read(#expr_range)
                        }
                    }
                });
            }
            if field.perm == BitFieldPerm::W || field.perm == BitFieldPerm::RW {
                current.extend(quote! {
                    impl ::bits::WriteField<#field_name> for #c_name {
                        fn write(&mut self, field: #field_name, v: #target_ty) {
                            ::bits::BitsMut(&mut self.0).write(#expr_range, v.into());
                        }
                    }
                });
            }
            if field.need_try == true {
                current.extend(quote! {
                    impl ::bits::TryReadField<#field_name> for #c_name {
                        type Error = <#target_ty as TryFrom<Self::BaseType>>::Error;
                        fn try_read(&self, field: #field_name) -> Result<#target_ty, Self::Error> {
                            ::bits::Bits(self.0).read(#expr_range).try_into()
                        }
                    }
                });
            }
            field_scop.extend(current);
        }
        let base_ty = &self.base_ty;
        let vis = &self.vis;
        let doc = &self.doc;
        let raw_vis = &self.export;
        let top_scop = quote! {
            #(#doc)*
            #vis struct #c_name(#raw_vis #base_ty);
            impl ::bits::Bitalized for #c_name {
                type BaseType = #base_ty;
            }
            pub mod field {
                use super::*;
                #field_scop
            }
        }; // 这里我们不更改 ident 的命名风格，否则会对 rust-analyzer 等 lint 工具产生误导。
        tokens.extend(top_scop);
    }
}
