use crate::plugin::Plugin;

use crate::{Struct, Method};
use std::fs::File;
use std::path::Path;
use std::io::Write;
use syn::ReturnType;
use quote::quote;

struct HeaderExporter {
    output_dir: String,
}

#[allow(dead_code)]
impl HeaderExporter {
    fn new(output_dir: &str) -> HeaderExporter {
        HeaderExporter {
            output_dir: String::from(output_dir),
        }
    }

    fn write(self: &Self, text: &str) -> std::io::Result<()> {
        let mut file = File::with_options()
            .create(true).append(true)
            .open(Path::new(&self.output_dir).join("api.h")).unwrap();
        file.write_all(text.as_bytes())
    }

    fn writeln(self: &Self, text: &str) -> std::io::Result<()>{
        self.write(&(text.to_owned() + "\n"))
    }
}

impl Plugin for HeaderExporter {
    fn on_struct(self: &Self, struct_target: &Struct) {
        let decl = format!("typedef void* {}", struct_target.id);
        self.writeln(&decl)
            .expect(&format!("Failed to write to {}/api.h", self.output_dir));
    }

    fn on_impl(self: &Self, methods: &Vec<Method>) {
        methods.iter().for_each(|method| {
            let return_ty = return_type(method);
            let fn_name = method.exported_name();
            let args : Vec<String> = method.exported_args().iter().map(|a| {
                format!("{} {}", a.1, a.0)
            }).collect();
            let args = args.join(", ");
            let decl = quote! {
                #return_ty #fn_name(#args);
            };
            self.writeln(&decl.to_string())
                .expect(&format!("Failed to write to {}/api.h", self.output_dir));
        });
    }
}

fn return_type(method: &Method) -> String {
    match &method.method.sig.output {
        ReturnType::Type(_, ty) => quote!{ #ty }.to_string(),
        ReturnType::Default => String::from("void")
    }
}
