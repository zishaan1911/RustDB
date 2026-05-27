// Parser nodes - create_index.rs placeholder
//! CREATE INDEX statement AST node

pub struct CreateIndexNode {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: IndexType,
}

pub enum IndexType {
    BTree,
    Hash,
    Composite,
}
