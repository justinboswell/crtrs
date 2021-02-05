
extern crate proc_macro;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, Ident, ImplItem, ImplItemMethod, Item, ItemImpl, ItemStruct, ReturnType, Type, FnArg, Pat};
use quote::{quote, format_ident, ToTokens};

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
    output.into()
}

enum Target {
    Struct(Struct),
    Impl(Vec<Method>),
}

struct Struct {
    id: Ident,
}

struct Method {
    is_static: bool,
    target: Struct,
    method: ImplItemMethod,
}

fn parse_target(macro_target: Item) -> Result<Target, &'static str> {
    return match macro_target {
        Item::Struct(struct_item) => Ok(Target::Struct(parse_struct(&struct_item))),
        Item::Impl(impl_item) => Ok(Target::Impl(parse_impl(impl_item))),
        _ => Err("crt_export attached to unknown token, only applicable to struct or impl")
    }
}

fn parse_struct(struct_item: &ItemStruct) -> Struct {
    return Struct {
        id: struct_item.ident.clone()
    }
}

fn parse_impl(impl_item: ItemImpl) -> Vec<Method> {
    let mut methods: Vec<Method> = vec![];
    for item in impl_item.items.iter() {
        if let ImplItem::Method(method) = item {
            methods.push(Method {
                is_static: method_is_static(method),
                target: Struct {
                    id: impl_target(&impl_item).unwrap().clone()
                },
                method: method.clone(),
            });
        }
    }
    methods
}

fn method_is_static(method: &ImplItemMethod) -> bool {
    match method.sig.inputs.first() {
        Some(FnArg::Receiver(..)) => false,
        _ => true,
    }
}

fn impl_target(impl_item: &ItemImpl) -> Result<&Ident, &'static str> {
    let struct_type = &impl_item.self_ty;
    return if let Type::Path(struct_path) = struct_type.as_ref() {
        Ok(struct_path.path.get_ident().unwrap())
    } else {
        Err("No struct found in target item")
    }
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

fn export_static_method(method: &Method) -> TokenStream {
    let fn_name = &method.method.sig.ident;
    let exported_fn = format_ident!("{}_{}", method.target.id, method.method.sig.ident);
    let args = export_args(&method.target.id, &method.method);
    let arg_names = arg_names(&method.method);
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
    let args = export_args(&method.target.id, &method.method);
    let arg_names = arg_names(&method.method);
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
            let this = unsafe { me.as_ref().expect("NULL self provided") };
            #return_kw this.#fn_name(#arg_names);
        }
    };
    //println!("code: \n{}", gen);
    gen.into()
}

fn export_drop_method(method: &Method) -> TokenStream {
    let exported_fn = format_ident!("{}_{}", method.target.id, method.method.sig.ident);
    let args = export_args(&method.target.id, &method.method);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn #exported_fn(#args) {
            let this = unsafe { me.as_ref().expect("NULL self provided") };
            std::mem::drop(this);
        }
    };
    //println!("code: \n{}", gen);
    gen.into()
}

fn arg_names(method: &ImplItemMethod) -> TokenStream {
    let mut args = String::new();
    method.sig.inputs.pairs().for_each(|p| {
       match p.value() {
           FnArg::Receiver(..) => (),
           FnArg::Typed(typed) => match &typed.pat.as_ref() {
               Pat::Ident(ident) => {
                   if !args.is_empty() {
                       args.push_str(", ");
                   }
                   args.push_str(&ident.ident.to_string())
               },
               _ => panic!("What")
           }
       }
    });
    syn::parse_str(&args).unwrap()
}

fn export_args(struct_ident: &Ident, method: &ImplItemMethod) -> TokenStream {
    let is_static = method_is_static(method);
    let mut inputs : Vec<&FnArg> = method.sig.inputs.pairs().map(|p| {
        *p.value()
    }).collect();
    if !is_static {
        if inputs.len() > 1 {
            inputs = inputs[1..].to_owned();
        } else {
            inputs = vec![];
        }
    }
    let mut args = String::new();

    // Convert self: &mut Self -> me: *mut Self
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
    //println!("args: ({})", arg_tokens);
    arg_tokens
}

