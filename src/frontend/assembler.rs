use std::rc::Rc;

use crate::{
    ErrorInfo, Func, FuncType, INITIALIZER_METHOD_NAME, Instruction, MAX_FUNCTION_ARGUMENTS, Value,
    frontend::{
        Token, TokenType,
        compiler::{Compiler, Local},
        rule::Precedence,
        scanner::TokenSource,
    },
    utils::word_to_bytes,
};

type ParseRule = super::rule::ParseRule<Assembler>;
type ClassCompiler = ();

pub struct Assembler {
    current: Token,
    previous: Token,
    compiler: Option<Box<Compiler>>,
    scanner: Box<dyn TokenSource>,
    panic_mode: bool,
    errors: Vec<ErrorInfo>,
    loop_stack: Vec<LoopData>,
    class_compilers: Vec<ClassCompiler>,
}

impl Assembler {
    pub fn new(scanner: Box<dyn TokenSource>) -> Self {
        Self {
            current: Token::undefined(),
            previous: Token::undefined(),
            compiler: None,
            scanner,
            panic_mode: false,
            errors: Vec::new(),
            loop_stack: Vec::new(),
            class_compilers: Vec::new(),
        }
    }

    pub fn compile(mut self) -> Result<Func, Vec<ErrorInfo>> {
        self.init_compiler(FuncType::Script);
        self.advance();
        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }
        let func = self.end_compiler().function_consumed();

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(func)
    }

    fn advance(&mut self) {
        self.update_previous();
        let mut looping = true;
        while looping {
            let token = self.scanner.scan_token();
            let is_err = token.is_err();
            self.set_current(token);
            if is_err {
                self.error_at_current("");
            }
            looping = is_err;
        }
    }

    fn is_match(&mut self, t_type: TokenType) -> bool {
        if !self.check(t_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, t_type: TokenType) -> bool {
        self.cur_token_type() == t_type
    }

    fn declaration(&mut self) {
        if self.is_match(TokenType::Class) {
            self.class_declaration();
        } else if self.is_match(TokenType::Fun) {
            self.fun_declaration();
        } else if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while !matches!(self.cur_token_type(), TokenType::Eof) {
            if matches!(self.prev_token_type(), TokenType::Semicolon) {
                return;
            }
            match self.cur_token_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn consume<T: AsRef<str>>(&mut self, t_type: TokenType, message: T) {
        if self.cur_token_type() == t_type {
            self.advance();
            return;
        };

        self.error_at_current(message.as_ref());
    }

    fn end_compiler(&mut self) -> Compiler {
        self.emit_return();
        let Some(mut compiler) = self.compiler.take() else {
            panic!("Can't end compiler which is None")
        };
        self.compiler = compiler.enclosing.take();
        *compiler
    }
}

/// Functions
impl Assembler {
    fn init_compiler(&mut self, func_type: FuncType) {
        let mut compiler = Compiler::with(func_type, self.compiler.take());
        if !matches!(func_type, FuncType::Script) {
            compiler.assign_name(self.prev_token_text());
        }
        self.compiler = Some(Box::new(compiler));
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name");
        self.compiler_mut().mark_initialized();
        self.function(FuncType::Function);
        self.define_variable(global);
    }

    fn function(&mut self, func_type: FuncType) {
        self.init_compiler(func_type);
        self.begin_scope();

        self.consume(TokenType::LeftParenthesis, "Expect '(' after function name");
        if !self.check(TokenType::RightParenthesis) {
            loop {
                self.compiler_mut().function_mut().arity += 1;
                if self.compiler().function().arity > MAX_FUNCTION_ARGUMENTS {
                    self.error_at_current("Can't have more than 255 parameters");
                }
                let constant = self.parse_variable("Expect parameter name");
                self.define_variable(constant);

                if !self.is_match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParenthesis, "Expect ')' after parameters");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body");
        self.block();

        let compiler = self.end_compiler();
        let (func, upvalues) = compiler.consume_closure_data();
        let upvalues_count = func.upvalue_count;

        let idx = self.make_constant(Value::Fun(Rc::new(func)));
        self.emit_instruction(&Instruction::Closure(idx));
        let line = self.get_line();
        upvalues
            .iter()
            .take(upvalues_count)
            .map(|data| data.as_vec())
            .for_each(|buffer| {
                self.compiler_mut().emit_buffer(&buffer, line);
            });
    }
}

/// expressions
impl Assembler {
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let t_type = self.prev_token_type();
        let Some(prefix_rule) = self.get_rule(t_type).prefix else {
            self.error("Expect expression");
            return;
        };

        let can_assign = precedence.le(&Precedence::Assignment);
        prefix_rule(self, can_assign);

        while precedence.le(&self.get_rule(self.cur_token_type()).precedence) {
            self.advance();
            let infix_rule = self
                .get_rule(self.prev_token_type())
                .infix
                .expect("Infix is none");
            infix_rule(self, can_assign);
        }

        if can_assign && self.is_match(TokenType::Equal) {
            self.error("Invalid assignment target");
        }
    }

    fn get_rule(&self, t_type: TokenType) -> ParseRule {
        use TokenType::*;
        match t_type {
            LeftParenthesis => {
                ParseRule::new(Some(Self::grouping), Some(Self::call), Precedence::Call)
            }
            Minus => ParseRule::new(Some(Self::unary), Some(Self::binary), Precedence::Term),
            Plus => ParseRule::new(None, Some(Self::binary), Precedence::Term),
            Slash | Star => ParseRule::new(None, Some(Self::binary), Precedence::Factor),
            Number => ParseRule::new(Some(Self::number), None, Precedence::None),
            Nil | False | True => ParseRule::new(Some(Self::literal), None, Precedence::None),
            Bang => ParseRule::new(Some(Self::unary), None, Precedence::None),
            EqualEqual | BangEqual => {
                ParseRule::new(None, Some(Self::binary), Precedence::Equality)
            }
            Greater | GreaterEqual | Less | LessEqual => {
                ParseRule::new(None, Some(Self::binary), Precedence::Comparison)
            }
            TokenType::String => ParseRule::new(Some(Self::string), None, Precedence::None),
            Identifier => ParseRule::new(Some(Self::variable), None, Precedence::None),
            And => ParseRule::new(None, Some(Self::and), Precedence::And),
            Or => ParseRule::new(None, Some(Self::or), Precedence::Or),
            Dot => ParseRule::new(None, Some(Self::dot), Precedence::Call),
            This => ParseRule::new(Some(Self::this), None, Precedence::None),
            _ => Default::default(),
        }
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list() as u8;
        self.emit_instruction(&Instruction::Call(arg_count));
    }

    fn argument_list(&mut self) -> usize {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParenthesis) {
            loop {
                self.expression();
                if arg_count == MAX_FUNCTION_ARGUMENTS {
                    self.error("Can't have more than 255 arguments");
                }
                arg_count += 1;
                if !self.is_match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParenthesis, "Expect ')' after arguments");
        arg_count
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.prev_token_type();
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.increased());

        let array: &[Instruction] = match operator_type {
            TokenType::BangEqual => &[Instruction::Equal, Instruction::Not],
            TokenType::EqualEqual => &[Instruction::Equal],
            TokenType::Greater => &[Instruction::Greater],
            TokenType::GreaterEqual => &[Instruction::Less, Instruction::Not],
            TokenType::Less => &[Instruction::Less],
            TokenType::LessEqual => &[Instruction::Greater, Instruction::Not],
            TokenType::Plus => &[Instruction::Add],
            TokenType::Minus => &[Instruction::Subtract],
            TokenType::Star => &[Instruction::Multiply],
            TokenType::Slash => &[Instruction::Divide],
            x => unreachable!("Unexpected binary operator {x:?}"),
        };
        self.emit_instructions(array);
    }

    fn dot(&mut self, can_assign: bool) {
        self.consume(TokenType::Identifier, "Expect property name after '.'");
        let name = self.identifier_constant(self.previous.clone());

        if can_assign && self.is_match(TokenType::Equal) {
            self.expression();
            self.emit_instruction(&Instruction::SetProperty(name));
        } else {
            self.emit_instruction(&Instruction::GetProperty(name));
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after expression");
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.prev_token_type() {
            TokenType::False => self.emit_instruction(&Instruction::False),
            TokenType::True => self.emit_instruction(&Instruction::True),
            TokenType::Nil => self.emit_instruction(&Instruction::Nil),
            _ => unreachable!("literal"),
        };
    }

    fn number(&mut self, _can_assign: bool) {
        // I don't like this approach
        // according to strtod it returns 0.0 as fallback
        let value = Value::number_from(self.prev_token_text()).unwrap_or(Value::Number(0.0));
        self.emit_constant(value);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        let end_jump = self.emit_instruction(&Instruction::stub_jump());

        self.patch_jump(else_jump);
        self.emit_instruction(&Instruction::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn string(&mut self, _can_assign: bool) {
        let s = self.prev_token_text();
        let text = &s[1..s.len() - 1];
        self.emit_constant(Value::text_from_str(text));
    }

    fn this(&mut self, _can_assign: bool) {
        if self.class_compilers.is_empty() {
            self.error("Can't use 'this' outside of a class");
            return;
        }
        self.variable(false);
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.prev_token_type();
        // Compile the operand
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction
        match operator_type {
            TokenType::Minus => self.emit_instruction(&Instruction::Negate),
            TokenType::Bang => self.emit_instruction(&Instruction::Not),
            _ => unreachable!("unary"),
        };
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.prev_token_owned(), can_assign);
    }

    fn named_variable(&mut self, token: Token, can_assign: bool) {
        let (getter, setter) = if let Some(info) = self.compiler().resolve_local(&token) {
            if info.depth.is_none() {
                self.error("Can't read local variable in its own initializer");
            }
            (
                Instruction::GetLocal(info.index),
                Instruction::SetLocal(info.index),
            )
        } else if let Some(index) = self.resolve_upvalue(&token) {
            (
                Instruction::GetUpvalue(index),
                Instruction::SetUpvalue(index),
            )
        } else {
            let idx = self.identifier_constant(token);
            (Instruction::GetGlobal(idx), Instruction::SetGlobal(idx))
        };
        if can_assign && self.is_match(TokenType::Equal) {
            self.expression();
            self.emit_instruction(&setter);
        } else {
            self.emit_instruction(&getter);
        }
    }

    fn resolve_upvalue(&mut self, token: &Token) -> Option<u8> {
        match self.compiler_mut().resolve_upvalue(token) {
            super::compiler::UpvalueResolve::NotFound => None,
            super::compiler::UpvalueResolve::Index(index) => Some(index),
            super::compiler::UpvalueResolve::Error(err) => {
                self.error(err);
                Some(0)
            }
        }
    }
}

/// Variables
impl Assembler {
    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name");

        if self.is_match(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_instruction(&Instruction::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(global);
    }

    fn parse_variable(&mut self, message: &str) -> u8 {
        self.consume(TokenType::Identifier, message);
        self.declare_variable();
        if self.compiler().is_local_scope() {
            return 0;
        }
        self.identifier_constant(self.prev_token_owned())
    }

    fn declare_variable(&mut self) {
        if self.compiler().is_global_scope() {
            return;
        }
        let token = self.prev_token_owned();
        if self.compiler().has_declared_variable(&token) {
            self.error("Already a variable with this name in this scope");
        }
        self.add_local(token);
    }

    fn add_local(&mut self, token: Token) {
        if !self.compiler().has_capacity() {
            self.error("Too many local variables in function");
            return;
        }
        let local = Local::with_name(token.text);
        self.compiler_mut().push_local(local);
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler().is_local_scope() {
            self.compiler_mut().mark_initialized();
            return;
        }
        self.emit_instruction(&Instruction::DefineGlobal(global));
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        self.make_constant(Value::text_from_string(token.text))
    }
}

/// Classes
impl Assembler {
    fn class_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect class name");
        let class_name = self.prev_token_owned();
        let idx = self.identifier_constant(self.prev_token_owned());
        self.declare_variable();

        self.emit_instruction(&Instruction::Class(idx));
        self.define_variable(idx);

        self.class_compilers.push(());

        self.named_variable(class_name, false);
        self.consume(TokenType::LeftBrace, "Expect '{' before class body");
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.method();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body");
        self.emit_instruction(&Instruction::Pop);

        self.class_compilers.pop();
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect method name");
        let idx = self.identifier_constant(self.prev_token_owned());

        let func_type = if self.previous.text == INITIALIZER_METHOD_NAME {
            FuncType::Initializer
        } else {
            FuncType::Method
        };
        self.function(func_type);

        self.emit_instruction(&Instruction::Method(idx));
    }
}

/// Statements
impl Assembler {
    fn statement(&mut self) {
        if self.is_match(TokenType::Print) {
            self.print_statement();
            return;
        }
        if self.is_match(TokenType::Break) {
            self.break_statement();
            return;
        }
        if self.is_match(TokenType::Continue) {
            self.continue_statement();
            return;
        }
        if self.is_match(TokenType::For) {
            self.for_statement();
            return;
        }
        if self.is_match(TokenType::If) {
            self.if_statement();
            return;
        }
        if self.is_match(TokenType::Return) {
            self.return_statement();
            return;
        }
        if self.is_match(TokenType::Switch) {
            self.switch_statement();
            return;
        }
        if self.is_match(TokenType::While) {
            self.while_statement();
            return;
        }
        if self.is_match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
            return;
        }
        self.expression_statement();
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Pop);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'if'");
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after condition");

        let then_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.statement();

        let else_jump = self.emit_instruction(&Instruction::stub_jump());

        self.patch_jump(then_jump);
        self.emit_instruction(&Instruction::Pop);

        if self.is_match(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    /// Function implemented according to the book's grammar:
    /// switchStmt     → "switch" "(" expression ")"
    ///                  "{" switchCase* defaultCase? "}" ;
    /// switchCase     → "case" expression ":" statement* ;
    /// defaultCase    → "default" ":" statement* ;
    ///
    /// this isn't mainstream approach to use expressions in switch cases
    /// because their values are not known at compile time.
    /// As result, it may lead to unexpected behavior when
    /// different case entries will be associated with the same value
    fn switch_statement(&mut self) {
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'if'");
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after condition");
        self.consume(TokenType::LeftBrace, "Expect '{' after 'switch' statement");

        let mut exit_jumps: Vec<usize> = Vec::new();
        let mut default_offset: Option<usize> = None;
        loop {
            if self.is_match(TokenType::Case) {
                self.emit_instruction(&Instruction::Duplicate);
                self.expression();
                self.consume(TokenType::Colon, "Expect ':' after case expression");
                self.emit_instruction(&Instruction::Equal);
                let next_case = self.emit_instruction(&Instruction::stub_jump_if_false());
                // remove compare result for true/match case
                self.emit_instruction(&Instruction::Pop);
                self.switch_branch_statement();
                let exit_jump = self.emit_instruction(&Instruction::stub_jump());
                exit_jumps.push(exit_jump);
                // remove compare result for false case
                self.patch_jump(next_case);
                self.emit_instruction(&Instruction::Pop);
            } else if self.is_match(TokenType::DefaultCase) {
                if default_offset.is_some() {
                    self.error("Multiple default labels in one switch");
                }
                self.consume(TokenType::Colon, "Expect ':' after default case");
                // jump to end-of-default block
                let default_exit_jump = self.emit_instruction(&Instruction::stub_jump());
                default_offset = Some(self.chunk_position());
                self.switch_branch_statement();
                let exit_jump = self.emit_instruction(&Instruction::stub_jump());
                exit_jumps.push(exit_jump);
                self.patch_jump(default_exit_jump);
            } else {
                break;
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after 'switch' block");
        if let Some(offset) = default_offset {
            self.emit_loop(offset);
        }
        exit_jumps
            .into_iter()
            .for_each(|offset| self.patch_jump(offset));
    }

    fn switch_branch_statement(&mut self) {
        self.emit_instruction(&Instruction::Pop);
        loop {
            match self.cur_token_type() {
                TokenType::Case
                | TokenType::DefaultCase
                | TokenType::RightBrace
                | TokenType::Eof => break,
                _ => self.statement(),
            }
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.mark_start_loop();
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'while'");
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after condition");

        let exit_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_instruction(&Instruction::Pop);
        self.flush_loop();
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'for'");
        if self.is_match(TokenType::Semicolon) {
            // no initializer
        } else if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.mark_start_loop();
        let mut exit_jump: Option<usize> = None;
        if !self.is_match(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");
            exit_jump = Some(self.emit_instruction(&Instruction::stub_jump_if_false()));
            self.emit_instruction(&Instruction::Pop);
        }

        if !self.is_match(TokenType::RightParenthesis) {
            let body_jump = self.emit_instruction(&Instruction::stub_jump());
            let increment_start = self.chunk_position();
            self.expression();
            self.emit_instruction(&Instruction::Pop);
            self.consume(TokenType::RightParenthesis, "Expect ')' after for clauses");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_instruction(&Instruction::Pop); // condition
        }

        self.flush_loop();
        self.end_scope();
    }

    fn break_statement(&mut self) {
        self.consume(TokenType::Semicolon, "Expect ';' after 'break'");
        if self.loop_stack.is_empty() {
            self.error("'break' statement allowed inside loops only");
        }
        let offset = self.emit_instruction(&Instruction::stub_jump());
        self.emit_instruction(&Instruction::Pop);
        let Some(data) = self.loop_stack.last_mut() else {
            self.error("Bug: loop stack became empty");
            return;
        };
        data.breaks.push(offset);
    }

    fn continue_statement(&mut self) {
        self.consume(TokenType::Semicolon, "Expect ';' after 'continue'");
        let Some(data) = self.loop_stack.last() else {
            self.error("'continue' statement allowed inside loops only");
            return;
        };
        self.emit_loop(data.start);
    }

    fn mark_start_loop(&mut self) -> usize {
        let start = self.chunk_position();
        let data = LoopData::new(start);
        self.loop_stack.push(data);
        start
    }

    fn flush_loop(&mut self) {
        let Some(val) = self.loop_stack.pop() else {
            self.error("Bug: loop_stack is broken");
            return;
        };
        for exit_jump in val.breaks {
            self.patch_jump(exit_jump);
        }
    }

    fn begin_scope(&mut self) {
        self.compiler_mut().begin_scope();
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn end_scope(&mut self) {
        let line = self.get_line();
        self.compiler_mut().end_scope(line);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Print);
    }

    fn return_statement(&mut self) {
        if matches!(self.compiler().func_type(), FuncType::Script) {
            self.error("Can't return from top-level code");
        }

        if self.is_match(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value");
            self.emit_instruction(&Instruction::Return);
        }
    }
}

/// Emit functions
impl Assembler {
    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.compiler_mut().add_constant(value);
        if idx > u8::MAX as usize {
            self.error("Too many constants in one chunk");
            // don't think it's a good decision
            // but this index seems doesn't reachable
            return 0;
        }
        idx as u8
    }

    fn emit_constant(&mut self, value: Value) {
        let idx = self.make_constant(value);
        self.emit_instruction(&Instruction::Constant(idx));
    }

    fn emit_return(&mut self) -> usize {
        let instruction = match self.compiler().func_type() {
            FuncType::Initializer => Instruction::GetLocal(0),
            _ => Instruction::Nil,
        };
        self.emit_instruction(&instruction);
        self.emit_instruction(&Instruction::Return)
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> usize {
        let line = self.get_line();
        self.compiler_mut()
            .emit_instruction_at_line(instruction, line)
    }

    fn emit_instructions(&mut self, instruction: &[Instruction]) {
        instruction
            .iter()
            .for_each(|inst| _ = self.emit_instruction(inst));
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let instr = Instruction::Loop(0x0, 0x0);
        let size = instr.size();
        let offset = self.chunk_position() - loop_start + size;
        if offset > u16::MAX as usize {
            self.error("Jump size is too large");
        }
        let (f, s) = word_to_bytes(offset);
        self.emit_instruction(&Instruction::Loop(f, s));
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let (fetch_result, size) = self.compiler().fetch_instruction(offset);

        let jump = self.chunk_position() - offset - size;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over");
        }
        let (first, second) = word_to_bytes(jump);
        let instr = match fetch_result {
            Ok(Instruction::JumpIfFalse(_, _)) => Instruction::JumpIfFalse(first, second),
            Ok(Instruction::Jump(_, _)) => Instruction::Jump(first, second),
            Err(err) => {
                self.error(&format!("Bug: {err}"));
                return;
            }
            _ => {
                self.error("Bug: Attempt to patch non-jump instruction in 'path_jump' function");
                return;
            }
        };
        self.compiler_mut().patch_instruction(&instr, offset);
    }

    fn chunk_position(&self) -> usize {
        self.compiler().chunk_position()
    }
}

// Errors
impl Assembler {
    fn error_at_current(&mut self, message: &str) {
        self.push_error_info(self.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.push_error_info(self.prev_token_owned(), message);
    }

    // convenience function
    fn push_error_info(&mut self, elem: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let info = ErrorInfo::with(elem, message);
        self.errors.push(info);
    }
}

/// Shorthands
impl Assembler {
    // Actually isn't great solution but it looks a critical issue if compiler is None
    // so no reason to continue execution and panic is acceptable behavior.
    // Will redesign by propagating result to each Assembler's function
    fn compiler(&self) -> &Compiler {
        self.compiler.as_ref().expect("Bug: compiler can't be None")
    }

    fn compiler_mut(&mut self) -> &mut Box<Compiler> {
        self.compiler.as_mut().expect("Bug: compiler can't be None")
    }

    fn prev_token_owned(&self) -> Token {
        self.previous.clone()
    }

    fn prev_token_text(&self) -> &str {
        &self.previous.text
    }

    fn set_current(&mut self, token: Token) {
        self.current = token;
    }

    fn update_previous(&mut self) {
        self.previous = self.current.clone();
    }

    fn prev_token_type(&self) -> TokenType {
        self.previous.t_type
    }

    fn cur_token_type(&self) -> TokenType {
        self.current.t_type
    }

    fn get_line(&self) -> usize {
        self.previous.position.line
    }
}

struct LoopData {
    start: usize,
    breaks: Vec<usize>,
}

impl LoopData {
    fn new(start: usize) -> Self {
        Self {
            start,
            breaks: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::scanner::tests::ScannerMock;

    use super::*;

    #[test]
    fn advance_test_normal() {
        let mut frontend = compose_frontend_with_tokens(vec![Token::minus(), Token::number("123")]);
        frontend.advance();
        assert_eq!(frontend.previous, Token::undefined());
        assert_eq!(frontend.current, Token::minus());
    }

    #[test]
    fn advance_test_error() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::error("wrong"), Token::number("123")]);
        assert!(!frontend.panic_mode);
        frontend.advance();
        assert_eq!(frontend.previous, Token::undefined());
        assert_eq!(frontend.current, Token::number("123"));
        assert!(frontend.panic_mode);
    }

    #[test]
    fn match_test_true() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::plus(), Token::minus(), Token::number("123")]);
        // call advance to fill initial undefined token's value
        frontend.advance();
        assert!(frontend.is_match(TokenType::Plus));
        assert_eq!(frontend.previous, Token::plus());
        assert_eq!(frontend.current, Token::minus());
    }

    #[test]
    fn match_test_false() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::plus(), Token::minus(), Token::number("123")]);
        // call advance to fill initial undefined token's value
        frontend.advance();
        assert!(!frontend.is_match(TokenType::False));
        assert_eq!(frontend.previous, Token::undefined());
        assert_eq!(frontend.current, Token::plus());
    }

    #[test]
    fn token_shorthands_test() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::plus(), Token::minus(), Token::number("123")]);
        frontend.advance();
        frontend.advance();
        assert_eq!(frontend.prev_token_type(), frontend.previous.t_type);
        assert_eq!(frontend.cur_token_type(), frontend.current.t_type);

        assert_eq!(frontend.prev_token_type(), TokenType::Plus);
        assert_eq!(frontend.cur_token_type(), TokenType::Minus);

        assert_eq!(frontend.prev_token_owned(), frontend.previous);
    }

    //
    fn compose_frontend_with_tokens(tokens: Vec<Token>) -> Assembler {
        let scanner = ScannerMock::new(tokens);
        compose_frontend(Box::new(scanner))
    }

    fn compose_frontend(scanner: Box<dyn TokenSource>) -> Assembler {
        Assembler::new(scanner)
    }

    // legacy test group

    #[test]
    fn emit_unary_chunk() {
        let input = vec![Token::minus(), Token::number("12.345"), Token::semicolon()];
        let expectation = Expectation {
            constants: vec![Value::number(12.345)],
            instructions: vec![Instruction::Constant(0), Instruction::Negate],
        };
        state_expectation_test(input, expectation);
    }

    #[test]
    fn emit_binary_chunk() {
        let data = [
            (
                Token::make(TokenType::BangEqual, "!="),
                vec![Instruction::Equal, Instruction::Not],
            ),
            (
                Token::make(TokenType::EqualEqual, "=="),
                vec![Instruction::Equal],
            ),
            (
                Token::make(TokenType::Greater, ">"),
                vec![Instruction::Greater],
            ),
            (
                Token::make(TokenType::GreaterEqual, ">="),
                vec![Instruction::Less, Instruction::Not],
            ),
            (Token::make(TokenType::Less, "<"), vec![Instruction::Less]),
            (
                Token::make(TokenType::LessEqual, "<="),
                vec![Instruction::Greater, Instruction::Not],
            ),
            (Token::minus(), vec![Instruction::Subtract]),
            (Token::plus(), vec![Instruction::Add]),
            (Token::multiply(), vec![Instruction::Multiply]),
            (Token::divide(), vec![Instruction::Divide]),
        ];
        for (token, expected_instr) in data {
            let input = vec![
                Token::number("3"),
                token,
                Token::number("5.0"),
                Token::semicolon(),
            ];
            let mut instructions = vec![Instruction::Constant(0), Instruction::Constant(1)];

            for exp_instr in expected_instr {
                instructions.push(exp_instr.clone());
            }
            let expectation = Expectation {
                constants: vec![Value::number(3.0), Value::number(5.0)],
                instructions,
            };
            state_expectation_test(input, expectation);
        }
    }

    #[test]
    fn emit_literal_chunk() {
        let tokens = vec![
            (Token::make(TokenType::False, "false"), Instruction::False),
            (Token::make(TokenType::True, "true"), Instruction::True),
            (Token::make(TokenType::Nil, "nil"), Instruction::Nil),
        ];
        for (token, emitted) in tokens {
            let expectation = Expectation {
                constants: Vec::new(),
                instructions: vec![emitted],
            };
            state_expectation_test(vec![token, Token::semicolon()], expectation);
        }
    }

    #[test]
    fn emit_grouping_chunk() {
        // 3 * (5 + 7)
        let input = vec![
            Token::number("3"),
            Token::multiply(),
            Token::with_type(TokenType::LeftParenthesis),
            Token::number("5.0"),
            Token::plus(),
            Token::number("7"),
            Token::with_type(TokenType::RightParenthesis),
            Token::semicolon(),
        ];

        let expectation = Expectation {
            constants: vec![Value::number(3.0), Value::number(5.0), Value::number(7.0)],
            instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Constant(2),
                Instruction::Add,
                Instruction::Multiply,
            ],
        };

        state_expectation_test(input, expectation);
    }

    #[test]
    fn emit_string_constant() {
        let input = vec![
            Token::make(TokenType::String, "\"Text\""),
            Token::semicolon(),
        ];
        let expectation = Expectation {
            constants: vec![Value::text_from_str("Text")],
            instructions: vec![],
        };
        state_expectation_test(input, expectation);
    }

    fn state_expectation_test(input: Vec<Token>, expectation: Expectation) {
        let mock = ScannerMock::new(input);
        let parser = Assembler::new(Box::new(mock));
        let compiler = parser.compile().expect("Failed to perform expectation");

        for (i, x) in expectation.constants.iter().enumerate() {
            assert_eq!(compiler.chunk().read_const(i as u8), Some(x.clone()));
        }

        let mut offset = 0;
        for instr_exp in expectation.instructions {
            let instr = compiler
                .chunk()
                .fetch(&mut offset)
                .expect("Failed to fetch instruction");
            assert_eq!(instr, instr_exp);
        }
    }

    struct Expectation {
        constants: Vec<Value>,
        instructions: Vec<Instruction>,
    }
}
