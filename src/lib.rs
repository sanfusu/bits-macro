use parse::{container_attr::BitContainerAttr, BitField, BitStructInfo};
use proc_macro::TokenStream;
use quote::quote;

mod parse;
///
/// /// export 表示是否设置 A(u32) 中位置元素为 public。及
/// A(pub u32)
/// #[bits(u32, export)]
/// struct A {
///     /// Punctuated<Expr, Token![,]>
///     #[field(pos=0..3, rw)]
///     pub field1: F1;
///     #[pos(4..7)]
///     #[perm(w)]
///     pub field2: F2;
///     #[pos(8..32)]
///     /// 私有字段，且默认权限为读写
///     field4: u32;
/// }
/// 当没有 export 的时候，需要自行实现读取函数
#[proc_macro_attribute]
pub fn bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    let c_info = syn::parse::<BitContainerAttr>(attr);
    if let Err(x) = c_info {
        return x.to_compile_error().into();
    }
    let b_info = syn::parse::<BitStructInfo>(item);
    if let Err(err) = b_info {
        return err.to_compile_error().into();
    }
    let bit_field = BitField::new(c_info.unwrap(), b_info.unwrap());
    quote! (#bit_field).into()
}
