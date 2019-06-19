
mod init;
pub mod scan;
pub mod codegen;

use ::serde;
use ::serde_derive;

/// Simplified AST only for supported interface features.
/// It's easier to process on this simplified AST.
/// As Rust AST changes constantly, it's better to have this one.

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum Item {
    Module(Module),
    Enum(Enum),
    Struct(Struct),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}
/// An enum variant always have non-empty name.
/// An enum variant can have tuple or struct fields.
/// If a variant has tuple fields, all fields should have empty name.
/// If a variant has struct fields, all fields should have non-empty name.
///
/// In Swift-side, tuple field will become name-less parameters.
/// Struct fields will become named paramers.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum Type {
    Core(CoreType),
    String,
    // Subtype name.
    // Usually an `enum` or `struct`.
    Item(TypePath),
    Option(Box<Type>),
    Vec(Box<Type>),
    //HashMap(Box<Type>,Box<Type>),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum CoreType {
    Bool,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    /*
    D32, D64,
    */
}

type Variant = EnumVariant;
type Field = StructField;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct TypePath(Vec<String>);

