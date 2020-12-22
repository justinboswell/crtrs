
extern crate proc_macro;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use syn::{
    parse_macro_input, Ident, ImplItem, ImplItemMethod, Item, ItemImpl, ItemStruct, Type 
};
use quote::{quote, format_ident};

#[proc_macro_attribute]
pub fn crt_export(_attr: RawTokenStream, tokens: RawTokenStream) -> RawTokenStream {
    let original = tokens.clone();
    let target = parse_macro_input!(tokens as Item);
    //println!("TARGET={:#?}", target);
    let mut output : RawTokenStream = match target {
        Item::Struct(struct_item) => export_struct(struct_item).into(),
        Item::Impl(impl_item) => export_impl(impl_item).into(),
        _ => RawTokenStream::new()
    };
    output.extend(original);
    output.into()
}

fn export_struct(struct_item: ItemStruct) -> TokenStream {
    let struct_name = struct_item.ident;
    let new_fn = format_ident!("{}_new", struct_name);
    let del_fn = format_ident!("{}_destroy", struct_name);
    let gen = quote!{
        extern crate libc;
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #new_fn() -> *mut #struct_name {
            unsafe { std::mem::zeroed() } 
        }
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #del_fn(s: *mut #struct_name) {
            std::mem::drop(s);
        }
        #[repr(C)]
    };
    gen.into()
}

fn export_impl(impl_item: ItemImpl) -> TokenStream {
    let struct_type = *impl_item.self_ty;
    
    let mut gen_tokens = TokenStream::new();
    if let Type::Path(struct_path) = struct_type {
        let struct_ident = struct_path.path.get_ident().unwrap();
        for item in impl_item.items.iter() {
            gen_tokens.extend(match item {
                ImplItem::Method(method) => export_method(struct_ident, method),
                _ => TokenStream::new()
            })
        }
    }
    gen_tokens
}

fn export_method(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream { 
    let exported_fn = format_ident!("{}_{}", struct_ident, method.sig.ident);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #exported_fn() {
            
        }
    };
    gen.into()
}


