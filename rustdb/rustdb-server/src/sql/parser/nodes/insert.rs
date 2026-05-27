// Parser nodes - insert.rs placeholder
//! INSERT statement AST node

pub struct InsertNode {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Value>>,
}

pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}
