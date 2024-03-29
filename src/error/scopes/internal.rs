use crate::error::internal;

/* indicates something is wrong with the interpreter itself */
pub struct InternalError {
    msg: String,
}

impl InternalError {
    pub fn new(msg: &str) -> Self {
        InternalError {
            msg: msg.to_string(),
        }
    }
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", internal(&self.msg))
    }
}
