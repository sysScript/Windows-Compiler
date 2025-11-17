use std::fmt;

#[derive(Debug)]
pub enum CompilerError {
    LexerError(String),
    ParseError(String),
    SemanticError(String),
    CodeGenError(String),
    IoError(String),
    AssemblyError(String),
    LinkError(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            CompilerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CompilerError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            CompilerError::CodeGenError(msg) => write!(f, "Code generation error: {}", msg),
            CompilerError::IoError(msg) => write!(f, "IO error: {}", msg),
            CompilerError::AssemblyError(msg) => write!(f, "Assembly error: {}", msg),
            CompilerError::LinkError(msg) => write!(f, "Link error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}
