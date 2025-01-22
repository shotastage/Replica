// src/ast.rs
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
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub is_mutable: bool,
    pub ownership: OwnershipType,
}

#[derive(Debug)]
pub enum OwnershipType {
    Owned,
    Moved,
    Shared,
    Copied,
}
