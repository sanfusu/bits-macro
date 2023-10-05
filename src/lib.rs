use parse::{bits_attr::BitStructAttr, BitStruct, BitStructItem};
use proc_macro::TokenStream;
use quote::quote;

mod parse;
///
/// export 表示是否设置 A(u32) 中位置元素为 public。即 `A(pub(in module::path) u32)`
/// 因为有时候我们并不希望 A 能够被随意的创建，所以需要该参数。
/// ```
/// #[bits(u32, export::<module::path>)]
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
/// ```
/// 当没有 export 的时候，需要自行实现读取函数。
/// TODO: 有时候，target type 不是一个具体的类型，可能是一个 impl trait。
/// 比如 gpio 可能有 31 个，但是我们可复用的功能有限。一个功能可能只能在少数的 IO 口上使用。
/// 如果采用 enum 的形式，我们是没有办法在编译时判断其是否可用。
/// 那么采用 unit，然后实现各种 trait，
/// 以及某一个 IO 口上的 to_int 功能可以提供更准确的适用性。
/// ```
/// pub struct MuxFunc1;
/// pub struct MuxFunc2;
/// // MuxFunc1 在 Gpio1 和 Gpio2 的数值代表可能不一样。
/// impl ToGpio1Mux for MuxFunc1{}
/// impl ToGpio2Mux for MuxFunc1{}
/// // 只能传入 MuxFunc1。MuxFunc2 没有实现 ToGpio1Mux，也就是说 Gpio1 不能复用成 MuxFunc2.
/// pub fn gpio1_set_mux(v: impl ToGPio1Mux) {
/// }
/// ```
#[proc_macro_attribute]
pub fn bits(attr: TokenStream, item: TokenStream) -> TokenStream {
    let c_info = syn::parse::<BitStructAttr>(attr);
    if let Err(x) = c_info {
        return x.to_compile_error().into();
    }
    let b_info = syn::parse::<BitStructItem>(item);
    if let Err(err) = b_info {
        return err.to_compile_error().into();
    }
    let bit_field = BitStruct::new(c_info.unwrap(), b_info.unwrap());
    quote! (#bit_field).into()
}
