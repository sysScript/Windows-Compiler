use crate::error::CompilerError;
use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum AstNode {
    Module {
        name: String,
        items: Vec<AstNode>,
    },
    Function {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Vec<AstNode>,
    },
    VariableDecl {
        name: String,
        var_type: Option<Type>,
        value: Option<Box<AstNode>>,
        mutable: bool,
    },
    ConstDecl {
        name: String,
        const_type: Type,
        value: Box<AstNode>,
    },
    Return {
        value: Option<Box<AstNode>>,
    },
    BinaryOp {
        left: Box<AstNode>,
        op: String,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: String,
        operand: Box<AstNode>,
    },
    Literal(Literal),
    Identifier(String),
    FunctionCall {
        name: String,
        args: Vec<AstNode>,
    },
    If {
        condition: Box<AstNode>,
        then_branch: Vec<AstNode>,
        else_branch: Option<Vec<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
    },
    For {
        iterator: String,
        range_start: Box<AstNode>,
        range_end: Box<AstNode>,
        inclusive: bool,
        body: Vec<AstNode>,
    },
    Loop {
        body: Vec<AstNode>,
    },
    Break,
    Continue,
    Assignment {
        target: String,
        value: Box<AstNode>,
    },
    ArrayLiteral {
        elements: Vec<AstNode>,
    },
    ArrayRepeat {
        value: Box<AstNode>,
        count: usize,
    },
    ArrayIndex {
        array: Box<AstNode>,
        index: Box<AstNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    Void,
    Str,
    Array(Box<Type>, usize),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<AstNode, CompilerError> {
        let mut items = Vec::new();
        let module_name = self.parse_module_declaration()?;
        
        while !self.is_at_end() {
            if let Some(item) = self.parse_top_level()? {
                items.push(item);
            }
        }
        
        Ok(AstNode::Module {
            name: module_name,
            items,
        })
    }
    
    fn parse_module_declaration(&mut self) -> Result<String, CompilerError> {
        if self.match_token(&TokenType::Module) {
            if let TokenType::Identifier(name) = &self.current_token().token_type {
                let name = name.clone();
                self.advance();
                self.expect_token(&TokenType::Semicolon)?;
                Ok(name)
            } else {
                Err(CompilerError::ParseError("Expected module name".to_string()))
            }
        } else {
            Ok("main".to_string())
        }
    }
    
    fn parse_top_level(&mut self) -> Result<Option<AstNode>, CompilerError> {
        if self.match_token(&TokenType::Import) {
            self.parse_import()?;
            return Ok(None);
        }
        
        if self.match_token(&TokenType::Fn) {
            return Ok(Some(self.parse_function()?));
        }
        
        Err(CompilerError::ParseError(format!(
            "Unexpected token at top level: {:?}",
            self.current_token()
        )))
    }
    
    fn parse_import(&mut self) -> Result<(), CompilerError> {
        while !self.is_at_end() && !self.match_token(&TokenType::Semicolon) {
            self.advance();
        }
        Ok(())
    }
    
    fn parse_function(&mut self) -> Result<AstNode, CompilerError> {
        let name = if let TokenType::Identifier(n) = &self.current_token().token_type {
            n.clone()
        } else {
            return Err(CompilerError::ParseError("Expected function name".to_string()));
        };
        self.advance();
        
        self.expect_token(&TokenType::LeftParen)?;
        
        let mut params = Vec::new();
        while !self.check(&TokenType::RightParen) {
            let param_name = if let TokenType::Identifier(n) = &self.current_token().token_type {
                n.clone()
            } else {
                return Err(CompilerError::ParseError("Expected parameter name".to_string()));
            };
            self.advance();
            
            self.expect_token(&TokenType::Colon)?;
            let param_type = self.parse_type()?;
            
            params.push((param_name, param_type));
            
            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }
        
        self.expect_token(&TokenType::RightParen)?;
        
        let return_type = if self.match_token(&TokenType::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect_token(&TokenType::LeftBrace)?;
        
        let body = self.parse_block()?;
        
        self.expect_token(&TokenType::RightBrace)?;
        
        Ok(AstNode::Function {
            name,
            params,
            return_type,
            body,
        })
    }
    
    fn parse_block(&mut self) -> Result<Vec<AstNode>, CompilerError> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> Result<AstNode, CompilerError> {
        if self.match_token(&TokenType::Let) {
            return self.parse_variable_decl();
        }
        
        if self.match_token(&TokenType::Const) {
            return self.parse_const_decl();
        }
        
        if self.match_token(&TokenType::Return) {
            return self.parse_return();
        }
        
        if self.match_token(&TokenType::If) {
            return self.parse_if();
        }
        
        if self.match_token(&TokenType::While) {
            return self.parse_while();
        }
        
        if self.match_token(&TokenType::For) {
            return self.parse_for();
        }
        
        if self.match_token(&TokenType::Loop) {
            return self.parse_loop();
        }
        
        if self.match_token(&TokenType::Break) {
            self.expect_token(&TokenType::Semicolon)?;
            return Ok(AstNode::Break);
        }
        
        if self.match_token(&TokenType::Continue) {
            self.expect_token(&TokenType::Semicolon)?;
            return Ok(AstNode::Continue);
        }
        
        let expr = self.parse_expression()?;
        
        if self.match_token(&TokenType::Equal) {
            if let AstNode::Identifier(name) = expr {
                let value = self.parse_expression()?;
                self.expect_token(&TokenType::Semicolon)?;
                return Ok(AstNode::Assignment {
                    target: name,
                    value: Box::new(value),
                });
            }
        }
        
        self.expect_token(&TokenType::Semicolon)?;
        Ok(expr)
    }
    
    fn parse_variable_decl(&mut self) -> Result<AstNode, CompilerError> {
        let mutable = self.match_token(&TokenType::Mut);
        
        let name = if let TokenType::Identifier(n) = &self.current_token().token_type {
            n.clone()
        } else {
            return Err(CompilerError::ParseError("Expected variable name".to_string()));
        };
        self.advance();
        
        let var_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let value = if self.match_token(&TokenType::Equal) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        self.expect_token(&TokenType::Semicolon)?;
        
        Ok(AstNode::VariableDecl {
            name,
            var_type,
            value,
            mutable,
        })
    }
    
    fn parse_return(&mut self) -> Result<AstNode, CompilerError> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        self.expect_token(&TokenType::Semicolon)?;
        
        Ok(AstNode::Return { value })
    }
    
    fn parse_if(&mut self) -> Result<AstNode, CompilerError> {
        self.expect_token(&TokenType::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect_token(&TokenType::RightParen)?;
        
        self.expect_token(&TokenType::LeftBrace)?;
        let then_branch = self.parse_block()?;
        self.expect_token(&TokenType::RightBrace)?;
        
        let else_branch = if self.match_token(&TokenType::Else) {
            self.expect_token(&TokenType::LeftBrace)?;
            let else_body = self.parse_block()?;
            self.expect_token(&TokenType::RightBrace)?;
            Some(else_body)
        } else {
            None
        };
        
        Ok(AstNode::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn parse_while(&mut self) -> Result<AstNode, CompilerError> {
        self.expect_token(&TokenType::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect_token(&TokenType::RightParen)?;
        
        self.expect_token(&TokenType::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect_token(&TokenType::RightBrace)?;
        
        Ok(AstNode::While { condition, body })
    }
    
    fn parse_expression(&mut self) -> Result<AstNode, CompilerError> {
        self.parse_logical_or()
    }
    
    fn parse_logical_or(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_logical_and()?;
        
        while self.match_token(&TokenType::PipePipe) {
            let right = self.parse_logical_and()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: "||".to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_logical_and(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(&TokenType::AmpAmp) {
            let right = self.parse_equality()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: "&&".to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_comparison()?;
        
        while self.match_any(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let op = match &self.previous_token().token_type {
                TokenType::EqualEqual => "==",
                TokenType::NotEqual => "!=",
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_term()?;
        
        while self.match_any(&[TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual]) {
            let op = match &self.previous_token().token_type {
                TokenType::Less => "<",
                TokenType::LessEqual => "<=",
                TokenType::Greater => ">",
                TokenType::GreaterEqual => ">=",
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_term(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_factor()?;
        
        while self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let op = match &self.previous_token().token_type {
                TokenType::Plus => "+",
                TokenType::Minus => "-",
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_factor(&mut self) -> Result<AstNode, CompilerError> {
        let mut left = self.parse_unary()?;
        
        while self.match_any(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op = match &self.previous_token().token_type {
                TokenType::Star => "*",
                TokenType::Slash => "/",
                TokenType::Percent => "%",
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<AstNode, CompilerError> {
        if self.match_any(&[TokenType::Minus, TokenType::Bang]) {
            let op = match &self.previous_token().token_type {
                TokenType::Minus => "-",
                TokenType::Bang => "!",
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(AstNode::UnaryOp {
                op: op.to_string(),
                operand: Box::new(operand),
            });
        }
        
        self.parse_primary()
    }
    
    fn parse_primary(&mut self) -> Result<AstNode, CompilerError> {
        match &self.current_token().token_type {
            TokenType::IntLiteral(n) => {
                let val = *n;
                self.advance();
                Ok(AstNode::Literal(Literal::Int(val)))
            }
            TokenType::FloatLiteral(f) => {
                let val = *f;
                self.advance();
                Ok(AstNode::Literal(Literal::Float(val)))
            }
            TokenType::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Ok(AstNode::Literal(Literal::String(val)))
            }
            TokenType::CharLiteral(c) => {
                let val = *c;
                self.advance();
                Ok(AstNode::Literal(Literal::Char(val)))
            }
            TokenType::BoolLiteral(b) => {
                let val = *b;
                self.advance();
                Ok(AstNode::Literal(Literal::Bool(val)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                if self.check(&TokenType::LeftBracket) {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect_token(&TokenType::RightBracket)?;
                    return Ok(AstNode::ArrayIndex {
                        array: Box::new(AstNode::Identifier(name)),
                        index: Box::new(index),
                    });
                }
                
                if self.match_token(&TokenType::LeftParen) {
                    let mut args = Vec::new();
                    
                    while !self.check(&TokenType::RightParen) {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                    
                    self.expect_token(&TokenType::RightParen)?;
                    
                    Ok(AstNode::FunctionCall { name, args })
                } else {
                    Ok(AstNode::Identifier(name))
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(&TokenType::RightParen)?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                if self.check(&TokenType::RightBracket) {
                    self.advance();
                    return Ok(AstNode::ArrayLiteral { elements });
                }
                
                let first_expr = self.parse_expression()?;
                
                if self.match_token(&TokenType::Semicolon) {
                    if let TokenType::IntLiteral(count) = self.current_token().token_type {
                        self.advance();
                        self.expect_token(&TokenType::RightBracket)?;
                        return Ok(AstNode::ArrayRepeat {
                            value: Box::new(first_expr),
                            count: count as usize,
                        });
                    } else {
                        return Err(CompilerError::ParseError("Expected array size".to_string()));
                    }
                }
                
                elements.push(first_expr);
                
                while self.match_token(&TokenType::Comma) {
                    if self.check(&TokenType::RightBracket) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                }
                
                self.expect_token(&TokenType::RightBracket)?;
                Ok(AstNode::ArrayLiteral { elements })
            }
            _ => Err(CompilerError::ParseError(format!(
                "Unexpected token: {:?}",
                self.current_token()
            ))),
        }
    }
    
    fn parse_type(&mut self) -> Result<Type, CompilerError> {
        let ty = match &self.current_token().token_type {
            TokenType::I8 => Type::I8,
            TokenType::I16 => Type::I16,
            TokenType::I32 => Type::I32,
            TokenType::I64 => Type::I64,
            TokenType::U8 => Type::U8,
            TokenType::U16 => Type::U16,
            TokenType::U32 => Type::U32,
            TokenType::U64 => Type::U64,
            TokenType::F32 => Type::F32,
            TokenType::F64 => Type::F64,
            TokenType::Bool => Type::Bool,
            TokenType::Char => Type::Char,
            TokenType::Void => Type::Void,
            TokenType::Str => Type::Str,
            TokenType::LeftBracket => {
                self.advance();
                let element_type = self.parse_type()?;
                self.expect_token(&TokenType::Semicolon)?;
                
                if let TokenType::IntLiteral(size) = self.current_token().token_type {
                    self.advance();
                    self.expect_token(&TokenType::RightBracket)?;
                    return Ok(Type::Array(Box::new(element_type), size as usize));
                } else {
                    return Err(CompilerError::ParseError("Expected array size".to_string()));
                }
            }
            _ => return Err(CompilerError::ParseError("Expected type".to_string())),
        };
        self.advance();
        Ok(ty)
    }
    
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn match_any(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.current_token().token_type) == std::mem::discriminant(token_type)
    }
    
    fn expect_token(&mut self, token_type: &TokenType) -> Result<(), CompilerError> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(CompilerError::ParseError(format!(
                "Expected {:?}, got {:?}",
                token_type,
                self.current_token()
            )))
        }
    }
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous_token(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current_token().token_type, TokenType::Eof)
    }
    
    fn parse_const_decl(&mut self) -> Result<AstNode, CompilerError> {
        let name = if let TokenType::Identifier(n) = &self.current_token().token_type {
            n.clone()
        } else {
            return Err(CompilerError::ParseError("Expected constant name".to_string()));
        };
        self.advance();
        
        self.expect_token(&TokenType::Colon)?;
        let const_type = self.parse_type()?;
        
        self.expect_token(&TokenType::Equal)?;
        let value = Box::new(self.parse_expression()?);
        
        self.expect_token(&TokenType::Semicolon)?;
        
        Ok(AstNode::ConstDecl {
            name,
            const_type,
            value,
        })
    }
    
    fn parse_for(&mut self) -> Result<AstNode, CompilerError> {
        self.expect_token(&TokenType::LeftParen)?;
        
        let iterator = if let TokenType::Identifier(n) = &self.current_token().token_type {
            n.clone()
        } else {
            return Err(CompilerError::ParseError("Expected iterator variable".to_string()));
        };
        self.advance();
        
        if let TokenType::Identifier(kw) = &self.current_token().token_type {
            if kw != "in" {
                return Err(CompilerError::ParseError("Expected 'in' keyword".to_string()));
            }
        } else {
            return Err(CompilerError::ParseError("Expected 'in' keyword".to_string()));
        }
        self.advance();
        
        let range_start = Box::new(self.parse_expression()?);
        
        let inclusive = if self.match_token(&TokenType::Dot) {
            if self.match_token(&TokenType::Dot) {
                if self.match_token(&TokenType::Dot) {
                    true  // is ...
                } else {
                    false  // is ..
                }
            } else {
                return Err(CompilerError::ParseError("Expected range operator".to_string()));
            }
        } else {
            return Err(CompilerError::ParseError("Expected range operator".to_string()));
        };
        
        let range_end = Box::new(self.parse_expression()?);
        
        self.expect_token(&TokenType::RightParen)?;
        self.expect_token(&TokenType::LeftBrace)?;
        
        let body = self.parse_block()?;
        
        self.expect_token(&TokenType::RightBrace)?;
        
        Ok(AstNode::For {
            iterator,
            range_start,
            range_end,
            inclusive,
            body,
        })
    }
    
    fn parse_loop(&mut self) -> Result<AstNode, CompilerError> {
        self.expect_token(&TokenType::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect_token(&TokenType::RightBrace)?;
        
        Ok(AstNode::Loop { body })
    }
}