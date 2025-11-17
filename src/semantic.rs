use crate::error::CompilerError;
use crate::parser::{AstNode, Type, Literal};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct SymbolInfo {
    symbol_type: Type,
    mutable: bool,
}

pub struct SemanticAnalyzer {
    symbol_table: Vec<HashMap<String, SymbolInfo>>,
    current_function_return: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: vec![HashMap::new()],
            current_function_return: None,
        }
    }
    
    pub fn analyze(&mut self, ast: &AstNode) -> Result<(), CompilerError> {
        self.visit(ast)?;
        Ok(())
    }
    
    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }
    
    fn exit_scope(&mut self) {
        self.symbol_table.pop();
    }
    
    fn declare_variable(&mut self, name: String, var_type: Type, mutable: bool) -> Result<(), CompilerError> {
        if let Some(scope) = self.symbol_table.last_mut() {
            if scope.contains_key(&name) {
                return Err(CompilerError::SemanticError(
                    format!("Variable '{}' already declared in this scope", name)
                ));
            }
            scope.insert(name, SymbolInfo { symbol_type: var_type, mutable });
        }
        Ok(())
    }
    
    fn lookup_variable(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
    
    fn visit(&mut self, node: &AstNode) -> Result<Option<Type>, CompilerError> {
        match node {
            AstNode::Module { items, .. } => {
                for item in items {
                    self.visit(item)?;
                }
                Ok(None)
            }
            AstNode::Function { name: _, params, return_type, body } => {
                self.enter_scope();
                
                let old_return = self.current_function_return.clone();
                self.current_function_return = return_type.clone();
                
                for (param_name, param_type) in params {
                    self.declare_variable(param_name.clone(), param_type.clone(), false)?;
                }
                
                for stmt in body {
                    self.visit(stmt)?;
                }
                
                self.current_function_return = old_return;
                self.exit_scope();
                Ok(None)
            }
            AstNode::VariableDecl { name, var_type, value, mutable } => {
                let inferred_type = if let Some(val) = value {
                    self.visit(val)?
                } else {
                    None
                };
                
                let final_type = if let Some(explicit_type) = var_type {
                    if let Some(inf_type) = inferred_type {
                        if !self.types_compatible(explicit_type, &inf_type) {
                            return Err(CompilerError::SemanticError(
                                format!("Type mismatch: expected {:?}, got {:?}", explicit_type, inf_type)
                            ));
                        }
                    }
                    explicit_type.clone()
                } else if let Some(inf_type) = inferred_type {
                    inf_type
                } else {
                    return Err(CompilerError::SemanticError(
                        format!("Cannot infer type for variable '{}'", name)
                    ));
                };
                
                self.declare_variable(name.clone(), final_type, *mutable)?;
                Ok(None)
            }
            AstNode::ConstDecl { name, const_type, value } => {
                let value_type = self.visit(value)?;
                if let Some(val_type) = value_type {
                    if !self.types_compatible(const_type, &val_type) {
                        return Err(CompilerError::SemanticError(
                            format!("Constant type mismatch: expected {:?}, got {:?}", const_type, val_type)
                        ));
                    }
                }
                self.declare_variable(name.clone(), const_type.clone(), false)?;
                Ok(None)
            }
            AstNode::Return { value } => {
                if let Some(val) = value {
                    let return_type = self.visit(val)?;
                    if let Some(expected) = &self.current_function_return {
                        if let Some(actual) = return_type {
                            if !self.types_compatible(expected, &actual) {
                                return Err(CompilerError::SemanticError(
                                    format!("Return type mismatch: expected {:?}, got {:?}", expected, actual)
                                ));
                            }
                        }
                    }
                }
                Ok(None)
            }
            AstNode::BinaryOp { left, op, right } => {
                let left_type = self.visit(left)?;
                let right_type = self.visit(right)?;
                
                if let (Some(lt), Some(rt)) = (left_type, right_type) {
                    if !self.types_compatible(&lt, &rt) {
                        return Err(CompilerError::SemanticError(
                            format!("Type mismatch in binary operation: {:?} {} {:?}", lt, op, rt)
                        ));
                    }
                    
                    match op.as_str() {
                        "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => {
                            Ok(Some(Type::Bool))
                        }
                        _ => Ok(Some(lt))
                    }
                } else {
                    Ok(None)
                }
            }
            AstNode::UnaryOp { operand, .. } => {
                self.visit(operand)
            }
            AstNode::Literal(lit) => {
                Ok(Some(match lit {
                    Literal::Int(_) => Type::I32,
                    Literal::Float(_) => Type::F64,
                    Literal::String(_) => Type::Str,
                    Literal::Bool(_) => Type::Bool,
                    Literal::Char(_) => Type::Char,
                }))
            }
            AstNode::Identifier(name) => {
                if let Some(info) = self.lookup_variable(name) {
                    Ok(Some(info.symbol_type.clone()))
                } else {
                    Err(CompilerError::SemanticError(
                        format!("Undefined variable '{}'", name)
                    ))
                }
            }
            AstNode::FunctionCall { name: _, args } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(Some(Type::I32))
            }
            AstNode::If { condition, then_branch, else_branch } => {
                let cond_type = self.visit(condition)?;
                if let Some(t) = cond_type {
                    if t != Type::Bool {
                        return Err(CompilerError::SemanticError(
                            "Condition must be boolean".to_string()
                        ));
                    }
                }
                
                self.enter_scope();
                for stmt in then_branch {
                    self.visit(stmt)?;
                }
                self.exit_scope();
                
                if let Some(else_body) = else_branch {
                    self.enter_scope();
                    for stmt in else_body {
                        self.visit(stmt)?;
                    }
                    self.exit_scope();
                }
                
                Ok(None)
            }
            AstNode::While { condition, body } => {
                let cond_type = self.visit(condition)?;
                if let Some(t) = cond_type {
                    if t != Type::Bool {
                        return Err(CompilerError::SemanticError(
                            "Condition must be boolean".to_string()
                        ));
                    }
                }
                
                self.enter_scope();
                for stmt in body {
                    self.visit(stmt)?;
                }
                self.exit_scope();
                
                Ok(None)
            }
            AstNode::For { iterator, range_start, range_end, body, .. } => {
                self.visit(range_start)?;
                self.visit(range_end)?;
                
                self.enter_scope();
                self.declare_variable(iterator.clone(), Type::I32, false)?;
                
                for stmt in body {
                    self.visit(stmt)?;
                }
                self.exit_scope();
                
                Ok(None)
            }
            AstNode::Loop { body } => {
                self.enter_scope();
                for stmt in body {
                    self.visit(stmt)?;
                }
                self.exit_scope();
                Ok(None)
            }
            AstNode::Break | AstNode::Continue => {
                Ok(None)
            }
            AstNode::Assignment { target, value } => {
                let symbol_info = if let Some(info) = self.lookup_variable(target) {
                    info.clone()
                } else {
                    return Err(CompilerError::SemanticError(
                        format!("Undefined variable '{}'", target)
                    ));
                };
                
                if !symbol_info.mutable {
                    return Err(CompilerError::SemanticError(
                        format!("Cannot assign to immutable variable '{}'", target)
                    ));
                }
                
                let value_type = self.visit(value)?;
                if let Some(val_type) = value_type {
                    if !self.types_compatible(&symbol_info.symbol_type, &val_type) {
                        return Err(CompilerError::SemanticError(
                            format!("Type mismatch in assignment to '{}'", target)
                        ));
                    }
                }
                
                Ok(None)
            }
            AstNode::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    return Ok(Some(Type::Array(Box::new(Type::I32), 0)));
                }
                
                let first_type = self.visit(&elements[0])?;
                if let Some(elem_type) = first_type {
                    for elem in &elements[1..] {
                        let et = self.visit(elem)?;
                        if let Some(t) = et {
                            if !self.types_compatible(&elem_type, &t) {
                                return Err(CompilerError::SemanticError(
                                    "Array elements must have same type".to_string()
                                ));
                            }
                        }
                    }
                    Ok(Some(Type::Array(Box::new(elem_type), elements.len())))
                } else {
                    Ok(None)
                }
            }
            AstNode::ArrayRepeat { value, count } => {
                let elem_type = self.visit(value)?;
                if let Some(t) = elem_type {
                    Ok(Some(Type::Array(Box::new(t), *count)))
                } else {
                    Ok(None)
                }
            }
            AstNode::ArrayIndex { array, index } => {
                let array_type = self.visit(array)?;
                self.visit(index)?;
                
                if let Some(Type::Array(elem_type, _)) = array_type {
                    Ok(Some(*elem_type))
                } else {
                    Err(CompilerError::SemanticError(
                        "Can only index arrays".to_string()
                    ))
                }
            }
        }
    }
    
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        t1 == t2
    }
}