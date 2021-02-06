#![feature(with_options)]

mod header;
mod plugin;
mod types;

extern crate proc_macro;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, ImplItem, ImplItemMethod, Item, ItemImpl, ItemStruct, ReturnType};
use quote::{quote, format_ident, ToTokens};

use types::{Method, Struct};

#[proc_macro_attribute]
pub fn crt_export(_attr: RawTokenStream, tokens: RawTokenStream) -> RawTokenStream {
    let original = tokens.clone();
    let macro_target = parse_macro_input!(tokens as Item);
    let target = parse_target(macro_target).unwrap();

    let mut output : RawTokenStream = match target {
        Target::Struct(struct_target) => export_struct(&struct_target).into(),
        Target::Impl(impl_target) => export_impl(&impl_target).into(),
    };
    output.extend(original);
    // uncomment this if you get ICEs or panics during compilation and cargo expand won't
    // show you the source
    //println!("{}", output);
    output.into()
}

enum Target {
    Struct(Struct),
    Impl(Vec<Method>),
}

fn parse_target(macro_target: Item) -> Result<Target, &'static str> {
    return match macro_target {
        Item::Struct(struct_item) => Ok(Target::Struct(parse_struct(&struct_item))),
        Item::Impl(impl_item) => Ok(Target::Impl(parse_impl(impl_item))),
        _ => Err("crt_export attached to unknown token, only applicable to struct or impl")
    }
}

fn parse_struct(struct_item: &ItemStruct) -> Struct {
    Struct::new(&struct_item.ident)
}

fn parse_impl(impl_item: ItemImpl) -> Vec<Method> {
    let mut methods: Vec<Method> = vec![];
    let struct_target = Struct::new(types::impl_target(&impl_item).unwrap());
    for item in impl_item.items.iter() {
        if let ImplItem::Method(method) = item {
            methods.push(Method::new(&struct_target, method));
        }
    }
    methods
}

fn export_struct(_struct_target: &Struct) -> TokenStream {
    let gen = quote!{
        #[repr(C)]
    };
    gen.into()
}

fn export_impl(methods: &Vec<Method>) -> TokenStream {
    let mut gen_tokens = TokenStream::new();
    methods.iter().for_each(|method| {
        gen_tokens.extend(export_method(method))
    });
    gen_tokens
}

fn export_method(method: &Method) -> TokenStream {
    return if method.is_static {
        export_static_method(method)
    } else {
        export_self_method(method)
    }
}

fn export_return_type(method: &ImplItemMethod) -> TokenStream {
    match &method.sig.output {
        ReturnType::Default => TokenStream::new(),
        ReturnType::Type(_, ty) => quote! { -> #ty }
    }
}

fn export_rust_args(method: &Method) -> TokenStream {
    let mut args = TokenStream::new();
    method.args.iter().for_each(|a| {
        if !args.is_empty() {
            (quote! {, }).to_tokens(&mut args);
        }
        a.tokens.to_tokens(&mut args);
    });
    args.into()
}

fn export_rust_arg_refs(method: &Method) -> TokenStream {
    let mut args = TokenStream::new();
    let mut all_args = method.exported_args();
    if !method.is_static {
        all_args = if all_args.len() > 1 {
            all_args[1..].to_owned()
        } else {
            vec![]
        }
    }
    all_args.iter().for_each(|a| {
        if !args.is_empty() {
            (quote! {, }).to_tokens(&mut args);
        }
        let arg_ty = &a.1;
        let arg = format_ident!("{}", a.0);
        let arg_name = arg.to_string();
        if arg_ty.ends_with("*") {
            (quote! { unsafe { #arg.as_ref().expect(&format!("NULL provided for {}", #arg_name)) } }).to_tokens(&mut args);
        } else {
            arg.to_tokens(&mut args);
        }
    });
    args.into()
}

fn export_static_method(method: &Method) -> TokenStream {
    let fn_name = &method.method.sig.ident;
    let exported_fn = format_ident!("{}_{}", method.target.id, method.method.sig.ident);
    let args = export_rust_args(method);
    let arg_names = export_rust_arg_refs(method);
    let return_ty = export_return_type(&method.method);
    let return_kw = match return_ty.is_empty() {
        true => return_ty.clone(),
        false => quote! { return },
    };
    let target = &method.target.id;
    let gen = quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) #return_ty {
            #return_kw #target::#fn_name(#arg_names);
        }
    };
    //println!("code: \n{}", gen);
    gen.into()
}

fn export_self_method(method: &Method) -> TokenStream {
    let fn_name = &method.method.sig.ident;
    if fn_name == "drop" {
        return export_drop_method(method);
    }
    let exported_fn = format_ident!("{}_{}", method.target.id, method.method.sig.ident);
    let args = export_rust_args(method);
    let arg_names = export_rust_arg_refs(method);
    let return_ty = export_return_type(&method.method);
    let return_kw = match return_ty.is_empty() {
        true => return_ty.clone(),
        false => quote! { return },
    };
    let gen = quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) #return_ty {
            let this = unsafe { this.as_ref().expect("NULL self provided") };
            #return_kw this.#fn_name(#arg_names);
        }
    };
    //println!("code: \n{}", gen);
    gen.into()
}

fn export_drop_method(method: &Method) -> TokenStream {
    let exported_fn = format_ident!("{}_{}", method.target.id, method.method.sig.ident);
    let args = export_rust_args(method);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) {
            let this = unsafe { this.as_ref().expect("NULL self provided") };
            std::mem::drop(this);
        }
    };
    //println!("code: \n{}", gen);
    gen.into()
}

