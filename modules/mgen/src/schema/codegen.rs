
use super::Module;
use super::Struct;
use super::Enum;
use super::Item;
use super::Type;
use super::CoreType;
use super::TypePath;

impl Module {
    pub fn gen_swift_code(&self) -> String {
        let mut ss = Vec::<String>::new();
        for kitem in &self.items {
            ss.push(kitem.gen_swift_code());
        }
        return ss.join("\n\n\n");
    }
}
impl Item {
    pub fn gen_swift_code(&self) -> String {
        let mut s = String::new();
        match self {
            Item::Module(ref kmod) => {
                s.push_str(&kmod.gen_swift_code());
            },
            Item::Enum(ref kenum) => {
                s.push_str(&kenum.gen_swift_code());
            },
            Item::Struct(ref kstruct) => {
                s.push_str(&kstruct.gen_swift_code());
            },
        }
        return s;
    }
}
impl Enum {
    pub fn gen_swift_code(&self) -> String {
        let mut s = String::new();
        s.push_str("enum ");
        s.push_str(&self.name);
        s.push_str(": Codable {\n");
        s.push_str(&self.gen_members());
        s.push_str(&self.gen_decoding());
        s.push_str(&self.gen_encoding());
        s.push_str("}\n");
        return s;
    }
}
impl Struct {
    pub fn gen_swift_code(&self) -> String {
        let mut s = String::new();
        s.push_str("struct ");
        s.push_str(&self.name);
        s.push_str(": Codable {\n");
        s.push_str(&self.gen_members());
        s.push_str("}\n");
        return s;
    }
}










impl Enum {
    fn gen_members(&self) -> String {
        let mut s = String::new();
        for kvariant in &self.variants {
            s.push_str("    case ");
            s.push_str(&kvariant.name);
            if kvariant.fields.len() > 0 {
                s.push_str("(");
            }
            let mut ss = Vec::<String>::new();
            for kfield in &kvariant.fields {
                if kfield.name == "" {
                    ss.push(kfield.ty.gen_swift_code());
                }
                else {
                    ss.push([&kfield.name, ": ", &kfield.ty.gen_swift_code()].join(""));
                }
            }
            s.push_str(&ss.join(", "));
            if kvariant.fields.len() > 0 {
                s.push_str(")");
            }
            s.push_str("\n");
        }
        return s;
    }
    fn gen_encoding(&self) -> String {
        let mut s = String::new();
        s.push_str("    func encode(to encoder: Encoder) throws {\n");
        s.push_str("        var c = encoder.unkeyedContainer()\n");
        s.push_str("        switch self {\n");
        for kvariant in &self.variants {
            s.push_str("        case .");
            s.push_str(&kvariant.name);
            if kvariant.fields.len() > 0 {
                s.push_str("(");
            }
            let mut ss = Vec::<String>::new();
            for i in 0..kvariant.fields.len() {
                ss.push(["let f".to_string(), i.to_string()].join(""));
            }
            s.push_str(&ss.join(", "));
            if kvariant.fields.len() == 0 {
                s.push_str(":\n");
            }
            else { 
                s.push_str("):\n");
            }
            s.push_str("            try c.encode(\"");
            s.push_str(&kvariant.name);
            s.push_str("\")\n");
            for i in 0..kvariant.fields.len() {
                s.push_str("            try c.encode(f");
                s.push_str(&i.to_string());
                s.push_str(")\n");
            }
        }
        s.push_str("        }\n");
        s.push_str("    }\n");
        return s;
    }
    fn gen_decoding(&self) -> String {
        let mut s = String::new();
        s.push_str("    init(from decoder: Decoder) throws {\n");
        s.push_str("        var dec = try decoder.unkeyedContainer()\n");
        s.push_str("        let n = try dec.decode(String.self)\n");
        s.push_str("        switch n {\n");
        for kvariant in &self.variants {
            s.push_str("        case \"");
            s.push_str(&kvariant.name);
            s.push_str("\":\n");
            s.push_str("            self = .");
            s.push_str(&kvariant.name);
            if kvariant.fields.len() > 0 {
                s.push_str("(");
            }
            s.push_str("\n");
            let mut c = 0;
            for kfield in &kvariant.fields {
                c += 1;
                s.push_str("                try dec.decode(");
                s.push_str(&kfield.ty.gen_swift_code());
                s.push_str(".self)");
                let delimeter = if c < kvariant.fields.len() { "," } else { ")" };
                s.push_str(delimeter);
                s.push_str("\n");
            }
        }
        s.push_str("        default:\n");
        s.push_str("            preconditionFailure()\n");
        s.push_str("        }\n");
        s.push_str("    }\n");
        return s;
    }
}


impl Struct {
    fn gen_members(&self) -> String {
        let mut s = String::new();
        for kfield in &self.fields {
            s.push_str("    ");
            s.push_str("var "); 
            s.push_str(&kfield.name);
            s.push_str(&": ");
            s.push_str(&kfield.ty.gen_swift_code());
            s.push_str("\n");
        }
        return s;
    }
    fn gen_encoding(&self) -> String {
        let mut s = String::new();
        s.push_str("func toJSON(_ v: ");
        s.push_str(&self.name);
        s.push_str(") -> Any {\n");
        s.push_str("    return [\n");
        for ref field in &self.fields {
            s.push_str("            ");
            s.push_str("\"");
            s.push_str(&field.name);
            s.push_str("\"");
            s.push_str(": toJSON(v.");
            s.push_str(&field.name);
            s.push_str("),");
            s.push_str("\n");
        }
        s.push_str("    ] as [String: Any]\n");
        s.push_str("}\n");
        return s;
    }
    fn gen_decoding(&self) -> String {
        let mut s = String::new();
        s.push_str("func fromJSON(_ j: Any) -> ");
        s.push_str(&self.name);
        s.push_str(" {\n");
        s.push_str("    let o = j as! [String: Any]\n");
        s.push_str("    let r = ");
        s.push_str(&self.name);
        s.push_str("(\n");
        let mut s1s = Vec::<String>::new();
        for ref field in &self.fields {
            let mut s1 = String::new();
            s1.push_str("        ");
            s1.push_str(&field.name);
            s1.push_str(": fromJSON(o[\"");
            s1.push_str(&field.name);
            s1.push_str("\"]!)");
            s1s.push(s1);
        }
        s.push_str(&s1s.join(",\n"));
        s.push_str(")\n");
        s.push_str("    return r\n");
        s.push_str("}\n");
        return s;
    }
}

impl Type {
    pub fn gen_swift_code(&self) -> String {
        return match self {
            Type::Core(ref core_type) => {
                use self::CoreType::*;
                match core_type {
                    Bool => "Bool".to_string(),
                    U8 => "UInt8".to_string(),
                    U16 => "UInt16".to_string(),
                    U32 => "UInt32".to_string(),
                    U64 => "UInt64".to_string(),
                    I8 => "Int8".to_string(),
                    I16 => "Int16".to_string(),
                    I32 => "Int32".to_string(),
                    I64 => "Int64".to_string(),
                    F32 => "Float32".to_string(),
                    F64 => "Float64".to_string(),
                    /*
                    D32,
                    D64,
                    */
                }
            },
            Type::String => "String".to_string(),
            Type::Item(ref path) => path.gen_swift_code(),
            Type::Option(ref ty) => ["Optional<", &ty.gen_swift_code(), ">"].join(""),
            Type::Vec(ref ty) => ["Array<", &ty.gen_swift_code(), ">"].join(""),
        };
    }
}

impl TypePath {
    pub fn gen_swift_code(&self) -> String {
        return self.0.last().unwrap_or(&"????".to_string()).to_string();
    }
}
