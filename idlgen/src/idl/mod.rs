#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Definition {
    Module(Module),
    Struct(Struct),
    Enum(Enum),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: String,
    pub members: Vec<StructMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructMember {
    pub is_key: bool,
    pub datatype: Type,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Float,
    Double,

    Short,
    Long,
    LongLong,

    UnsignedShort,
    UnsignedLong,
    UnsignedLongLong,

    Char,
    WChar,
    Boolean,
    Octet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateType {
    Sequence(Box<Type>, Option<usize>),
    String(Option<usize>),
    WideString(Option<usize>),
    // FixedPoint(Option<(usize, usize)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    BaseType(BaseType),
    TemplateType(TemplateType),
    // ScopedName(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    // FixedPoint,
    Char(char),
    Bool(bool),
    String(String),
}
