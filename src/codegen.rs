use crate::error::CompilerError;
use crate::parser::{AstNode, Literal};
use std::collections::HashMap;

pub struct CodeGenerator {
    opt_level: u8,
    label_counter: usize,
    string_literals: Vec<String>,
    variables: HashMap<String, i32>,
    stack_offset: i32,
    loop_stack: Vec<(String, String)>, // (break_label, continue_label)
}

impl CodeGenerator {
    pub fn new(opt_level: u8) -> Self {
        CodeGenerator {
            opt_level,
            label_counter: 0,
            string_literals: Vec::new(),
            variables: HashMap::new(),
            stack_offset: 0,
            loop_stack: Vec::new(),
        }
    }
    
    pub fn generate(&mut self, ast: &AstNode) -> Result<String, CompilerError> {
        let mut output = String::new();
        self.generate_node(ast, &mut output)?;
        Ok(output)
    }
    
    pub fn to_assembly(&mut self, ast: &AstNode) -> Result<String, CompilerError> {
        self.string_literals.clear();
        self.variables.clear();
        self.stack_offset = 0;
        self.label_counter = 0;
        self.loop_stack.clear();
        
        let mut code = String::new();
        self.generate_assembly_node(ast, &mut code)?;
        
        let mut asm = String::new();
        
        asm.push_str("section .data\n");
        if !self.string_literals.is_empty() {
            for (i, s) in self.string_literals.iter().enumerate() {
                asm.push_str(&format!("    str_{}: db `{}`, 0\n", i, s.replace("\n", "\\n").replace("\r", "\\r")));
            }
        }
        asm.push_str("\n");
        asm.push_str("section .bss\n\n");
        asm.push_str("section .text\n");
        asm.push_str("    global main\n");
        asm.push_str("    extern ExitProcess\n");
        asm.push_str("    extern printf\n\n");
        
        asm.push_str(&code);
        
        Ok(asm)
    }
    
    fn generate_assembly_node(&mut self, node: &AstNode, asm: &mut String) -> Result<(), CompilerError> {
        match node {
            AstNode::Module { items, .. } => {
                for item in items {
                    self.generate_assembly_node(item, asm)?;
                }
            }
            AstNode::Function { name, body, .. } => {
                asm.push_str(&format!("{}:\n", name));
                asm.push_str("    push rbp\n");
                asm.push_str("    mov rbp, rsp\n");
                
                self.variables.clear();
                self.stack_offset = 0;
                
                let local_space = self.calculate_stack_space(body);
                let total_space = ((local_space + 32 + 15) / 16) * 16; // Align to 16 bytes + shadow space
                
                if total_space > 0 {
                    asm.push_str(&format!("    sub rsp, {}\n", total_space));
                }
                asm.push_str("\n");
                
                for stmt in body {
                    self.generate_statement(stmt, asm)?;
                }
                
                if !body.iter().any(|s| matches!(s, AstNode::Return { .. })) {
                    asm.push_str("    xor eax, eax\n");
                    asm.push_str("    leave\n");
                    asm.push_str("    ret\n");
                }
                
                asm.push_str("\n");
            }
            _ => {}
        }
        Ok(())
    }
    
    fn calculate_stack_space(&self, body: &[AstNode]) -> i32 {
        let mut count = 0;
        for stmt in body {
            match stmt {
                AstNode::VariableDecl { .. } | AstNode::ConstDecl { .. } => {
                    count += 1;
                }
                AstNode::For { body, .. } => {
                    count += 1; // for iterator variable
                    count += self.calculate_stack_space(body);
                }
                AstNode::While { body, .. } | AstNode::Loop { body } => {
                    count += self.calculate_stack_space(body);
                }
                AstNode::If { then_branch, else_branch, .. } => {
                    count += self.calculate_stack_space(then_branch);
                    if let Some(else_body) = else_branch {
                        count += self.calculate_stack_space(else_body);
                    }
                }
                _ => {}
            }
        }
        ((count * 8 + 15) / 16) * 16
    }
    
    fn generate_statement(&mut self, node: &AstNode, asm: &mut String) -> Result<(), CompilerError> {
        match node {
            AstNode::VariableDecl { name, value, .. } => {
                if let Some(val) = value {
                    self.generate_expression(val, asm)?;
                    
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), self.stack_offset);
                    asm.push_str(&format!("    mov [rbp-{}], rax\n", self.stack_offset));
                } else {
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), self.stack_offset);
                }
            }
            AstNode::ConstDecl { name, value, .. } => {
                self.generate_expression(value, asm)?;
                
                self.stack_offset += 8;
                self.variables.insert(name.clone(), self.stack_offset);
                asm.push_str(&format!("    mov [rbp-{}], rax\n", self.stack_offset));
            }
            AstNode::Return { value } => {
                if let Some(val) = value {
                    self.generate_expression(val, asm)?;
                } else {
                    asm.push_str("    xor eax, eax\n");
                }
                
                asm.push_str("    leave\n");
                asm.push_str("    ret\n");
            }
            AstNode::Assignment { target, value } => {
                self.generate_expression(value, asm)?;
                
                if let Some(&offset) = self.variables.get(target) {
                    asm.push_str(&format!("    mov [rbp-{}], rax\n", offset));
                }
            }
            AstNode::If { condition, then_branch, else_branch } => {
                let else_label = self.next_label();
                let end_label = self.next_label();
                
                self.generate_expression(condition, asm)?;
                asm.push_str("    test rax, rax\n");
                asm.push_str(&format!("    jz {}\n", else_label));
                
                for stmt in then_branch {
                    self.generate_statement(stmt, asm)?;
                }
                asm.push_str(&format!("    jmp {}\n", end_label));
                
                asm.push_str(&format!("{}:\n", else_label));
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.generate_statement(stmt, asm)?;
                    }
                }
                
                asm.push_str(&format!("{}:\n", end_label));
            }
            AstNode::While { condition, body } => {
                let start_label = self.next_label();
                let end_label = self.next_label();
                
                self.loop_stack.push((end_label.clone(), start_label.clone()));
                
                asm.push_str(&format!("{}:\n", start_label));
                self.generate_expression(condition, asm)?;
                asm.push_str("    test rax, rax\n");
                asm.push_str(&format!("    jz {}\n", end_label));
                
                for stmt in body {
                    self.generate_statement(stmt, asm)?;
                }
                
                asm.push_str(&format!("    jmp {}\n", start_label));
                asm.push_str(&format!("{}:\n", end_label));
                
                self.loop_stack.pop();
            }
            AstNode::For { iterator, range_start, range_end, inclusive, body } => {
                let start_label = self.next_label();
                let end_label = self.next_label();
                
                self.generate_expression(range_start, asm)?;
                self.stack_offset += 8;
                self.variables.insert(iterator.clone(), self.stack_offset);
                asm.push_str(&format!("    mov [rbp-{}], rax\n", self.stack_offset));
                
                self.generate_expression(range_end, asm)?;
                self.stack_offset += 8;
                let end_offset = self.stack_offset;
                asm.push_str(&format!("    mov [rbp-{}], rax\n", end_offset));
                
                self.loop_stack.push((end_label.clone(), start_label.clone()));
                
                asm.push_str(&format!("{}:\n", start_label));
                
                let iter_offset = *self.variables.get(iterator).unwrap();
                asm.push_str(&format!("    mov rax, [rbp-{}]\n", iter_offset));
                asm.push_str(&format!("    mov rcx, [rbp-{}]\n", end_offset));
                asm.push_str("    cmp rax, rcx\n");
                
                if *inclusive {
                    asm.push_str(&format!("    jg {}\n", end_label));
                } else {
                    asm.push_str(&format!("    jge {}\n", end_label));
                }
                
                for stmt in body {
                    self.generate_statement(stmt, asm)?;
                }
                
                asm.push_str(&format!("    mov rax, [rbp-{}]\n", iter_offset));
                asm.push_str("    inc rax\n");
                asm.push_str(&format!("    mov [rbp-{}], rax\n", iter_offset));
                
                asm.push_str(&format!("    jmp {}\n", start_label));
                asm.push_str(&format!("{}:\n", end_label));
                
                self.loop_stack.pop();
                self.stack_offset -= 8; // Clean up end value.
            }
            AstNode::Loop { body } => {
                let start_label = self.next_label();
                let end_label = self.next_label();
                
                self.loop_stack.push((end_label.clone(), start_label.clone()));
                
                asm.push_str(&format!("{}:\n", start_label));
                
                for stmt in body {
                    self.generate_statement(stmt, asm)?;
                }
                
                asm.push_str(&format!("    jmp {}\n", start_label));
                asm.push_str(&format!("{}:\n", end_label));
                
                self.loop_stack.pop();
            }
            AstNode::Break => {
                if let Some((break_label, _)) = self.loop_stack.last() {
                    asm.push_str(&format!("    jmp {}\n", break_label));
                }
            }
            AstNode::Continue => {
                if let Some((_, continue_label)) = self.loop_stack.last() {
                    asm.push_str(&format!("    jmp {}\n", continue_label));
                }
            }
            _ => {
                self.generate_expression(node, asm)?;
            }
        }
        Ok(())
    }
    
    fn generate_expression(&mut self, node: &AstNode, asm: &mut String) -> Result<(), CompilerError> {
        match node {
            AstNode::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        asm.push_str(&format!("    mov rax, {}\n", n));
                    }
                    Literal::Char(c) => {
                        asm.push_str(&format!("    mov rax, {}\n", *c as u32));
                    }
                    Literal::Bool(b) => {
                        asm.push_str(&format!("    mov rax, {}\n", if *b { 1 } else { 0 }));
                    }
                    Literal::String(s) => {
                        let index = self.string_literals.len();
                        self.string_literals.push(s.clone());
                        asm.push_str(&format!("    lea rax, [rel str_{}]\n", index));
                    }
                    _ => {}
                }
            }
            AstNode::Identifier(name) => {
                if let Some(&offset) = self.variables.get(name) {
                    asm.push_str(&format!("    mov rax, [rbp-{}]\n", offset));
                }
            }
            AstNode::BinaryOp { left, op, right } => {
                self.generate_expression(right, asm)?;
                asm.push_str("    push rax\n");
                
                self.generate_expression(left, asm)?;
                asm.push_str("    pop rcx\n");
                
                match op.as_str() {
                    "+" => asm.push_str("    add rax, rcx\n"),
                    "-" => asm.push_str("    sub rax, rcx\n"),
                    "*" => asm.push_str("    imul rax, rcx\n"),
                    "/" => {
                        asm.push_str("    xor rdx, rdx\n");
                        asm.push_str("    idiv rcx\n");
                    }
                    "%" => {
                        asm.push_str("    xor rdx, rdx\n");
                        asm.push_str("    idiv rcx\n");
                        asm.push_str("    mov rax, rdx\n");
                    }
                    "==" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    sete al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    "!=" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    setne al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    "<" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    setl al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    "<=" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    setle al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    ">" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    setg al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    ">=" => {
                        asm.push_str("    cmp rax, rcx\n");
                        asm.push_str("    setge al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    "&&" => {
                        asm.push_str("    and rax, rcx\n");
                    }
                    "||" => {
                        asm.push_str("    or rax, rcx\n");
                    }
                    _ => {}
                }
            }
            AstNode::UnaryOp { op, operand } => {
                self.generate_expression(operand, asm)?;
                match op.as_str() {
                    "-" => asm.push_str("    neg rax\n"),
                    "!" => {
                        asm.push_str("    test rax, rax\n");
                        asm.push_str("    setz al\n");
                        asm.push_str("    movzx rax, al\n");
                    }
                    _ => {}
                }
            }
            AstNode::FunctionCall { name, args } => {
                if name == "print" && !args.is_empty() {
                    if let AstNode::Literal(Literal::String(s)) = &args[0] {
                        let index = self.string_literals.len();
                        self.string_literals.push(format!("{}\n", s));
                        asm.push_str(&format!("    lea rcx, [rel str_{}]\n", index));
                    } else {
                        self.generate_expression(&args[0], asm)?;
                        asm.push_str("    mov rcx, rax\n");
                    }
                    asm.push_str("    sub rsp, 32\n");
                    asm.push_str("    call printf\n");
                    asm.push_str("    add rsp, 32\n");
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn generate_node(&mut self, node: &AstNode, output: &mut String) -> Result<(), CompilerError> {
        match node {
            AstNode::Module { name, items } => {
                output.push_str(&format!("; Module: {}\n", name));
                for item in items {
                    self.generate_node(item, output)?;
                }
            }
            AstNode::Function { name, params, return_type, body } => {
                output.push_str(&format!("function {}(", name));
                for (i, (param_name, param_type)) in params.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("{}: {:?}", param_name, param_type));
                }
                output.push_str(")");
                if let Some(ret_type) = return_type {
                    output.push_str(&format!(" -> {:?}", ret_type));
                }
                output.push_str(" {\n");
                
                self.variables.clear();
                self.stack_offset = 0;
                
                for stmt in body {
                    self.generate_node(stmt, output)?;
                }
                output.push_str("}\n\n");
            }
            _ => {}
        }
        Ok(())
    }
    
    fn next_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}