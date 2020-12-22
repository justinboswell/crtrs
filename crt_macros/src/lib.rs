
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::*;
use quote::{quote, format_ident};

#[proc_macro_attribute]
pub fn crt_export(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let original : TokenStream = tokens.clone();
    let target = parse_macro_input!(tokens as Item);
    //println!("TARGET={:#?}", target);
    let mut output = match target {
        Item::Struct(struct_item) => export_struct(struct_item),
        Item::Impl(impl_item) => export_impl(impl_item),
        _ => TokenStream::new()
    };
    output.extend(original);
    output
}

fn export_struct(struct_item: ItemStruct) -> TokenStream {
    let new_fn = format_ident!("{}_new", struct_item.ident);
    let del_fn = format_ident!("{}_destroy", struct_item.ident);
    let gen = quote!{
        extern crate libc;
        extern "C" fn #new_fn () {
            
        }
        extern "C" fn #del_fn () {

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
        extern "C" fn #exported_fn() {
            
        }
    };
    gen.into()
}


