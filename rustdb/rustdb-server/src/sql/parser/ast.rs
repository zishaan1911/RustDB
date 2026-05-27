// Parser placeholder - ast.rs
//! Abstract Syntax Tree definitions for SQL

#[derive(Debug, Clone)]
pub enum Statement {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    CreateIndex,
    BeginTransaction,
    Commit,
    Rollback,
    TrainModel,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal,
    Column,
    BinaryOp,
    FunctionCall,
    Subquery,
}
