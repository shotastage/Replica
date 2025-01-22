#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Custom(String),
    Array(Box<Type>),
    Optional(Box<Type>),
}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub ownership: OwnershipType,
}

#[derive(Debug)]
pub enum ActorType {
    Distributed,
    Single,
}

#[derive(Debug)]
pub struct Actor {
    pub name: String,
    pub actor_type: ActorType,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub is_async: bool,
    pub is_sequential: bool,
    pub is_immediate: bool,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Option<MethodBody>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub is_mutable: bool,
    pub ownership: OwnershipType,
}

#[derive(Debug, Clone)]
pub enum OwnershipType {
    Owned,
    Moved,
    Shared,
    Copied,
}

pub struct OwnershipInfo {
    pub ownership_type: OwnershipType,
    pub is_mutable: bool,
}

#[derive(Debug)]
pub enum Expression {
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Literal(LiteralValue),
    Variable(String),
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct MethodBody {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
    Expression(Expression),
}
