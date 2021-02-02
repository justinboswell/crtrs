
extern crate proc_macro;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, Ident, ImplItem, ImplItemMethod, Item, ItemImpl, ItemStruct, ReturnType, Type, FnArg};
use quote::{quote, format_ident, ToTokens};

#[proc_macro_attribute]
pub fn crt_export(_attr: RawTokenStream, tokens: RawTokenStream) -> RawTokenStream {
    let original = tokens.clone();
    let target = parse_macro_input!(tokens as Item);

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

fn method_is_static(method: &ImplItemMethod) -> bool {
    match method.sig.inputs.first() {
        Some(FnArg::Receiver(..)) => false,
        _ => true,
    }
}

fn export_method(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream { 
    match method_is_static(method) {
        true => export_static_method(struct_ident, method),
        false => export_self_method(struct_ident, method),
    }
}

fn export_return_type(method: &ImplItemMethod) -> TokenStream {
    match &method.sig.output {
        ReturnType::Default => TokenStream::new(),
        ReturnType::Type(_, ty) => quote! { -> #ty }
    }
}

fn export_static_method(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream {
    let fn_name = &method.sig.ident;
    let exported_fn = format_ident!("{}_{}", struct_ident, method.sig.ident);
    let args = export_args(struct_ident, method);
    let return_ty = export_return_type(method);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) #return_ty {
            #struct_ident::#fn_name();
        }
    };
    println!("code: \n{}", gen);
    gen.into()
}

fn export_self_method(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream {
    let fn_name = &method.sig.ident;
    let exported_fn = format_ident!("{}_{}", struct_ident, method.sig.ident);
    let args = export_args(struct_ident, method);
    let return_ty = export_return_type(method);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) #return_ty {
            let this = unsafe { me.as_ref().expect("NULL self provided") };
            this.#fn_name();
        }
    };
    gen.into()
}

fn export_args(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream {
    let is_static = method_is_static(method);
    let mut inputs : Vec<&FnArg> = method.sig.inputs.pairs().map(|p| {
        *p.value()
    }).collect();
    if !is_static && inputs.len() > 1 {
        inputs = inputs.split_off(1);
    }
    let mut args = String::new();
    if !is_static {
        let recv = quote! {
            me: *mut #struct_ident
        };
        args.push_str(&recv.to_string());
    }
    let params = inputs.iter().map(|p| {
        p.to_token_stream().to_string()
    });
    params.for_each(|p| {
        if !args.is_empty() {
            args.push_str(", ");
        }
        args.push_str(&p);
    });
    let arg_tokens = syn::parse_str(&args).unwrap();
    println!("args: ({})", arg_tokens);
    arg_tokens
}

