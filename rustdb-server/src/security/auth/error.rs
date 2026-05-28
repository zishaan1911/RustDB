use std::fmt;

#[derive(Debug)]
pub enum AuthError
{
    MissingHeader,
    InvalidFormat,
    InvalidKey,
    DatabaseError,
}

impl fmt::Display for AuthError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            AuthError::MissingHeader => write!(f, "Missing Authorization header"),
            AuthError::InvalidFormat => write!(f, "Invalid Authorization format"),
            AuthError::InvalidKey => write!(f, "Invalid API key"),
            AuthError::DatabaseError => write!(f, "Database error"),
        }
    }
}

impl std::error::Error for AuthError {}