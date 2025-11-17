use crate::error::CompilerError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Module, Import, Fn, Let, Mut, Const, Return, If, Else, While, For, Loop, Break, Continue,
    Struct, Enum, Union, Type, Pub, Unsafe, Defer,
    I8, I16, I32, I64, U8, U16, U32, U64, F32, F64, Bool, Char, Void, Str,
    
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),
    
    Identifier(String),
    
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    AmpAmp, PipePipe, Bang,
    Amp, Pipe, Caret, Tilde, LessLess, GreaterGreater,
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,
    Semicolon, Comma, Dot, Colon, ColonColon, Arrow, FatArrow,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.is_at_end() {
                tokens.push(Token {
                    token_type: TokenType::Eof,
                    line: self.line,
                    column: self.column,
                });
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, CompilerError> {
        let line = self.line;
        let column = self.column;
        
        let c = self.current_char();
        
        if c == '/' && self.peek() == Some('/') {
            self.skip_line_comment();
            return self.next_token();
        }
        
        if c == '/' && self.peek() == Some('*') {
            self.skip_block_comment()?;
            return self.next_token();
        }
        
        if c.is_ascii_digit() {
            return Ok(Token {
                token_type: self.read_number()?,
                line,
                column,
            });
        }
        
        if c.is_alphabetic() || c == '_' {
            return Ok(Token {
                token_type: self.read_identifier(),
                line,
                column,
            });
        }
        
        if c == '"' {
            return Ok(Token {
                token_type: self.read_string()?,
                line,
                column,
            });
        }
        
        if c == '\'' {
            return Ok(Token {
                token_type: self.read_char()?,
                line,
                column,
            });
        }
        
        let token_type = match c {
            '+' => { self.advance(); TokenType::Plus }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => { self.advance(); TokenType::Star }
            '/' => { self.advance(); TokenType::Slash }
            '%' => { self.advance(); TokenType::Percent }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::EqualEqual
                } else if self.current_char() == '>' {
                    self.advance();
                    TokenType::FatArrow
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Bang
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else if self.current_char() == '<' {
                    self.advance();
                    TokenType::LessLess
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else if self.current_char() == '>' {
                    self.advance();
                    TokenType::GreaterGreater
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    TokenType::AmpAmp
                } else {
                    TokenType::Amp
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::PipePipe
                } else {
                    TokenType::Pipe
                }
            }
            '^' => { self.advance(); TokenType::Caret }
            '~' => { self.advance(); TokenType::Tilde }
            '(' => { self.advance(); TokenType::LeftParen }
            ')' => { self.advance(); TokenType::RightParen }
            '{' => { self.advance(); TokenType::LeftBrace }
            '}' => { self.advance(); TokenType::RightBrace }
            '[' => { self.advance(); TokenType::LeftBracket }
            ']' => { self.advance(); TokenType::RightBracket }
            ';' => { self.advance(); TokenType::Semicolon }
            ',' => { self.advance(); TokenType::Comma }
            '.' => { self.advance(); TokenType::Dot }
            ':' => {
                self.advance();
                if self.current_char() == ':' {
                    self.advance();
                    TokenType::ColonColon
                } else {
                    TokenType::Colon
                }
            }
            _ => {
                return Err(CompilerError::LexerError(format!(
                    "Unexpected character '{}' at line {}, column {}",
                    c, line, column
                )));
            }
        };
        
        Ok(Token { token_type, line, column })
    }
    
    fn read_identifier(&mut self) -> TokenType {
        let mut s = String::new();
        
        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            s.push(self.current_char());
            self.advance();
        }
        
        match s.as_str() {
            "module" => TokenType::Module,
            "import" => TokenType::Import,
            "fn" => TokenType::Fn,
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "const" => TokenType::Const,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "loop" => TokenType::Loop,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "union" => TokenType::Union,
            "type" => TokenType::Type,
            "pub" => TokenType::Pub,
            "unsafe" => TokenType::Unsafe,
            "defer" => TokenType::Defer,
            "i8" => TokenType::I8,
            "i16" => TokenType::I16,
            "i32" => TokenType::I32,
            "i64" => TokenType::I64,
            "u8" => TokenType::U8,
            "u16" => TokenType::U16,
            "u32" => TokenType::U32,
            "u64" => TokenType::U64,
            "f32" => TokenType::F32,
            "f64" => TokenType::F64,
            "bool" => TokenType::Bool,
            "char" => TokenType::Char,
            "void" => TokenType::Void,
            "str" => TokenType::Str,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            _ => TokenType::Identifier(s),
        }
    }
    
    fn read_number(&mut self) -> Result<TokenType, CompilerError> {
        let mut num_str = String::new();
        let mut is_float = false;
        
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();
        }
        
        if !self.is_at_end() && self.current_char() == '.' {
            if self.peek() == Some('.') {
                // This is a range operator, not a float. return the integer we've read so far
                return Ok(TokenType::IntLiteral(num_str.parse().unwrap()));
            }
            
            is_float = true;
            num_str.push('.');
            self.advance();
            
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                num_str.push(self.current_char());
                self.advance();
            }
        }
        
        if is_float {
            Ok(TokenType::FloatLiteral(num_str.parse().unwrap()))
        } else {
            Ok(TokenType::IntLiteral(num_str.parse().unwrap()))
        }
    }
    
    fn read_string(&mut self) -> Result<TokenType, CompilerError> {
        self.advance();
        let mut s = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(CompilerError::LexerError("Unterminated string".to_string()));
                }
                match self.current_char() {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    _ => s.push(self.current_char()),
                }
            } else {
                s.push(self.current_char());
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(CompilerError::LexerError("Unterminated string".to_string()));
        }
        
        self.advance();
        Ok(TokenType::StringLiteral(s))
    }
    
    fn read_char(&mut self) -> Result<TokenType, CompilerError> {
        self.advance();
        
        if self.is_at_end() {
            return Err(CompilerError::LexerError("Unterminated character".to_string()));
        }
        
        let ch = if self.current_char() == '\\' {
            self.advance();
            match self.current_char() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                _ => self.current_char(),
            }
        } else {
            self.current_char()
        };
        
        self.advance();
        
        if self.is_at_end() || self.current_char() != '\'' {
            return Err(CompilerError::LexerError("Unterminated character".to_string()));
        }
        
        self.advance();
        Ok(TokenType::CharLiteral(ch))
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.position += 1;
                }
                _ => break,
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> Result<(), CompilerError> {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'
        
        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek() == Some('/') {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                return Ok(());
            }
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
                self.position += 1;
            } else {
                self.advance();
            }
        }
        
        Err(CompilerError::LexerError("Unterminated block comment".to_string()))
    }
    
    fn current_char(&self) -> char {
        self.input[self.position]
    }
    
    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}