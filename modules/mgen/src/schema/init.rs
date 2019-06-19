
use super::Module;
use super::Enum;
use super::EnumVariant;
use super::Struct;

impl Module {
    pub fn new() -> Module {
        return Module { name: String::new(), items: Vec::new() };
    }
}
impl Enum {
    pub fn new() -> Enum {
        return Enum { name: String::new(), variants: Vec::new() };
    }
}
impl EnumVariant {
    pub fn new() -> EnumVariant {
        return EnumVariant { name: String::new(), fields: Vec::new() };
    }
}
impl Struct {
    pub fn new() -> Struct {
        return Struct { name: String::new(), fields: Vec::new() } 
    }
    //pub fn a(&self) {
    //    self.gen_swift_code();
    //}
}


