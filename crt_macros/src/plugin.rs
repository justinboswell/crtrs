use crate::{Struct, Method};

pub trait Plugin {
    fn on_struct(&self, struct_target: &Struct);
    fn on_impl(&self, impl_target: &Vec<Method>);
}
