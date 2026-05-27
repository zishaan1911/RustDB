// Parser nodes - create_table.rs placeholder
//! CREATE TABLE statement AST node

pub struct CreateTableNode {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub constraints: Vec<Constraint>,
}

pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

pub enum DataType {
    Int,
    Bigint,
    Float,
    Double,
    String(usize),
    Decimal(usize, usize),
    Boolean,
    Timestamp,
}

pub enum Constraint {
    PrimaryKey(Vec<String>),
    UniqueKey(Vec<String>),
    ForeignKey {
        columns: Vec<String>,
        ref_table: String,
        ref_columns: Vec<String>,
    },
}
