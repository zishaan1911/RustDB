// Parser nodes - transaction.rs placeholder
//! Transaction control statements

pub enum TransactionStatement {
    Begin,
    Commit,
    Rollback,
    SavePoint(String),
    RollbackTo(String),
}
