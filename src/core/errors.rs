use actix::MailboxError;
use std::fmt;

pub struct AppError {
    pub message: String,
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl From<MailboxError> for AppError {
    fn from(error: MailboxError) -> Self {
        Self {
            message: format!("{}", error),
        }
    }
}
