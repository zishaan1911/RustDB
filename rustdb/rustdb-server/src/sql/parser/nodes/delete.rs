// Parser nodes - delete.rs placeholder
//! DELETE statement AST node

pub struct DeleteNode {
    pub table: String,
    pub where_clause: Option<Expression>,
}

pub struct Expression {
    // Expression details
}
