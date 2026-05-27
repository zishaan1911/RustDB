// Parser nodes - select.rs placeholder
//! SELECT statement AST node

pub struct SelectNode {
    pub columns: Vec<Column>,
    pub table: String,
    pub where_clause: Option<Expression>,
}

pub struct Column {
    pub name: String,
    pub alias: Option<String>,
}

pub struct Expression {
    // Expression details
}
