
use syn::{Ident, ImplItemMethod, ItemImpl, ReturnType, Type, FnArg, Pat};
use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};

#[derive(Clone)]
pub struct Struct {
    pub id: Ident,
}

impl Struct {
    pub fn new(ident: &Ident) -> Struct {
        return Struct {
            id: ident.clone(),
        }
    }

    pub fn exported_name(self: &Self) -> String {
        self.id.to_string()
    }
}

pub struct MethodArg {
    pub name: String,
    pub rust_type: String,
    pub c_type: String,
    pub tokens: TokenStream,
}

pub struct Method {
    pub is_static: bool,
    pub target: Struct,
    pub method: ImplItemMethod,
    pub this: Option<MethodArg>,
    pub args: Vec<MethodArg>,
}

#[allow(dead_code)]
impl Method {
    pub fn new(target: &Struct, method: &ImplItemMethod) -> Method {
        return Method {
            is_static: method_is_static(method),
            target: target.clone(),
            method: method.clone(),
            this: Method::parse_this(target, method),
            args: Method::parse_args(target, method),
        }
    }

    fn parse_this(target: &Struct, method: &ImplItemMethod) -> Option<MethodArg> {
        let is_static = method_is_static(method);
        return if is_static {
            None
        } else {
            // Convert self: &mut Self -> this: *mut Self
            let rust_type = &target.id;
            Some(MethodArg {
                name: String::from("this"),
                rust_type: format!("*mut {}", rust_type),
                c_type: String::from("void*"),
                tokens: quote! { this: *mut #rust_type },
            })
        }
    }

    fn parse_args(target: &Struct, method: &ImplItemMethod) -> Vec<MethodArg> {
        let mut args: Vec<MethodArg> = vec![];
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

        if !is_static {
            args.push(Method::parse_this(target, method).unwrap());
        }
        inputs.iter().for_each(|p| {
            if let FnArg::Typed(typed) = p {
                if let Pat::Ident(ident) = typed.pat.as_ref() {
                    let rust_ffi_ty = rust_to_ffi_type(&typed.ty);
                    args.push(MethodArg {
                        name: ident.ident.to_string(),
                        rust_type: rust_ffi_ty.to_string(),
                        c_type: rust_to_c_type(&typed.ty),
                        tokens: quote!{ #ident : #rust_ffi_ty }
                    })
                }
            }
        });

        args
    }

    pub fn exported_target_name(self: &Self) -> String {
        self.target.exported_name()
    }

    pub fn exported_name(self: &Self) -> String {
        return format_ident!("{}_{}", &self.target.exported_name(), &self.method.sig.ident).to_string();
    }

    pub fn exported_return_type(self: &Self) -> String {
        match &self.method.sig.output {
            ReturnType::Default => String::from("void"),
            ReturnType::Type(_, ty) => rust_to_c_type(ty)
        }
    }

    pub fn exported_args(self: &Self) -> Vec<(String, String)> {
        self.args.iter().map(|a| {
            (a.name.clone(), a.c_type.clone())
        }).collect()
    }
}

fn rust_to_ffi_type(ty: &Box<Type>) -> TokenStream {
    match ty.as_ref() {
        //Type::Array(array_ty) => format!("{}[]", array_ty.elem.as_ref().to_string()),
        Type::Path(ty_path) => return ty_path.path.to_token_stream(),
        Type::Verbatim(tokens) => return tokens.to_token_stream(),
        Type::Reference(ref_type) => {
            if let Type::Path(path) = ref_type.elem.as_ref() {
                let ty = path.path.get_ident().unwrap();
                return quote!{ *mut #ty }
            }
        },
        _ => ()
    }
    panic!("Unsupported FFI type: {}", ty.as_ref().to_token_stream().to_string())
}

fn rust_to_c_type(ty: &Box<Type>) -> String {
    match ty.as_ref() {
        //Type::Array(array_ty) => format!("{}[]", array_ty.elem.as_ref().to_string()),
        Type::Path(ty_path) => return ty_path.path.get_ident().unwrap().to_string(),
        Type::Verbatim(tokens) => return tokens.to_string(),
        Type::Reference(ref_type) => {
            if let Type::Path(path) = ref_type.elem.as_ref() {
                let ty = path.path.get_ident().unwrap();
                return (quote!{ #ty * }).to_string()
            }
        },
        _ => ()
    }
    panic!("Unsupported FFI type: {}", ty.as_ref().to_token_stream().to_string())
}

pub fn method_is_static(method: &ImplItemMethod) -> bool {
    match method.sig.inputs.first() {
        Some(FnArg::Receiver(..)) => false,
        _ => true,
    }
}

pub fn impl_target(impl_item: &ItemImpl) -> Result<&Ident, &'static str> {
    let struct_type = &impl_item.self_ty;
    return if let Type::Path(struct_path) = struct_type.as_ref() {
        Ok(struct_path.path.get_ident().unwrap())
    } else {
        Err("No struct found in target item")
    }
}
