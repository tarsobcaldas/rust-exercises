use std::{
    convert::TryFrom,
    error::Error,
    fmt,
    io::prelude::*,
    iter::Peekable,
    slice::Iter,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Token {
    Plus,
    Dash,
    Star,
    Slash,
    Caret,
    LeftParen,
    RightParen,
    End,
    Number(i64),
}

impl Token {
    fn is_binary(&self) -> bool {
        matches!(self, Token::Plus | Token::Dash | Token::Star | Token::Slash | Token::Caret)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Operator {
    Add,
    Multiply,
    Divide,
    Subtract,
    Power,
    Negative,
}

impl Operator {
    fn cmp_val(&self) -> usize {
        match self {
            Operator::Add => 1,
            Operator::Multiply => 3,
            Operator::Divide => 3,
            Operator::Subtract => 1,
            Operator::Power => 4,
            Operator::Negative => 2,
        }
    }
}

impl TryFrom<Token> for Operator {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Plus => Ok(Operator::Add),
            Token::Star => Ok(Operator::Multiply),
            Token::Dash => Ok(Operator::Subtract),
            Token::Caret => Ok(Operator::Power),
            Token::Slash => Ok(Operator::Divide),
            _ => Err("Token is not an operator"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Expression {
    Number(i64),
    Unary(Operator, Box<Expression>),
    Binary(Operator, Box<Expression>, Box<Expression>),
}

impl Expression {
    fn eval(&self) -> f64 {
        match self {
            Expression::Number(n) => *n as f64,
            Expression::Unary(_negative, expr) => -1_f64 * expr.eval(),
            Expression::Binary(Operator::Add, expr1, expr2) => expr1.eval() + expr2.eval(),
            Expression::Binary(Operator::Multiply, expr1, expr2) => expr1.eval() * expr2.eval(),
            Expression::Binary(Operator::Subtract, expr1, expr2) => expr1.eval() - expr2.eval(),
            Expression::Binary(Operator::Power, expr1, expr2) => {
                let expr1 = expr1.eval() as i64;
                let mut expr2 = expr2.eval() as i64;
                if expr2 < 0 {
                    expr2 *= -1;
                    println!("Negative numbers not allowed in exponents");
                }

                match expr1.checked_pow(expr2 as u32) {
                    Some(v) => v as f64,
                    None => {
                        eprintln!("{} ^ {} is too large", expr1, expr2);
                        0.0
                    }
                }
            }
            Expression::Binary(Operator::Divide, expr1, expr2) => expr1.eval() / expr2.eval(),
            _ => {
                panic!("Unreachable code: for expr {:?}", self);
            }
        }
    }
}


#[derive(Debug)]
struct SyntaxError {
    message: String,
    level: String,
}

impl SyntaxError {
    fn new_lex_error(message: String) -> Self {
        SyntaxError {
            message,
            level: "Lex".to_string(),
        }
    }

    fn new_parse_error(message: String) -> Self {
        SyntaxError {
            message,
            level: "Parse".to_string(),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} Error {}:", self.level, self.message)
    }
}

impl Error for SyntaxError {}


struct Parser<'a> {
    iter: &'a mut Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(iter: &'a mut Peekable<Iter<'a, Token>>) -> Self {
        Parser { iter }
    }

    fn assert_next(&mut self, token: Token) -> Result<(), SyntaxError> {
        let next = self.iter.next();
        if next.is_none() {
            return Err(SyntaxError::new_parse_error(
                "Unexpected end of input".to_string(),
            ));
        }

        if *next.unwrap() != token {
            return Err(SyntaxError::new_parse_error(format!(
                "Expected {:?}, found {:?}",
                token,
                next.unwrap()
            )));
        }

        Ok(())
    }

    fn primary(&mut self) -> Result<Expression, SyntaxError> {
        match self.iter.next().unwrap() {
            Token::Dash => {
                let op = Operator::Negative;
                let expr = self.expression(op.cmp_val())?;
                Ok(Expression::Unary(op, Box::new(expr)))
            }
            Token::LeftParen => {
                let expr = self.expression(0)?;
                self.assert_next(Token::RightParen)?;
                Ok(expr)
            }
            Token::Number(n) => Ok(Expression::Number(*n)),
            tok => Err(SyntaxError::new_parse_error(format!(
                "Unexpected token {:?}",
                tok
            ))),
        }
    }


    fn expression(&mut self, precedence: usize) -> Result<Expression, SyntaxError> {
        let mut expr = self.primary()?;
        while let Some(tok) = self.iter.peek() {
            if !tok.is_binary() {
                break;
            }
            let operator = Operator::try_from(**tok).unwrap();
            if operator.cmp_val() < precedence {
                break;
            }
            self.iter.next();
            let inner_precedence = match operator {
                Operator:: Power => operator.cmp_val(),
                _ => 1+ operator.cmp_val(),
            };
            let rhs = self.expression(inner_precedence)?;
            expr = Expression::Binary(operator, Box::new(expr), Box::new(rhs));
        }

        Ok(expr)
    }


    fn parse(&mut self) -> Result<Expression, SyntaxError> {
        let ast = self.expression(0)?;
        self.assert_next(Token::End)?;
        Ok(ast)
    }


}


fn lex(code: String) -> Result<Vec<Token>, SyntaxError> {
    let mut iter = code.chars().peekable();
    let mut tokens: Vec<Token> = Vec::new();
    let mut leftover: Option<char> = None;

    loop {
        let ch = match leftover {
            Some(ch) => ch,
            None => match iter.next() {
                None => break,
                Some(ch) => ch,
            },
        };
        leftover = None;

        match ch {
            ' ' => continue,
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Dash),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '^' => tokens.push(Token::Caret),
            ')' => tokens.push(Token::RightParen),
            '(' => tokens.push(Token::LeftParen),
            ch if ch.is_ascii_digit() => {
                let number_stream: String = iter
                    .by_ref()
                    .take_while(|c| match c.is_ascii_digit() {
                        true => true,
                        false => {
                            leftover = Some(*c);
                            false
                        }
                    })
                    .collect();
                let number: i64 = format!("{}{}", ch, number_stream).parse().unwrap();
                tokens.push(Token::Number(number));
            }

            _ => {
                return Err(SyntaxError::new_lex_error(format!(
                    "Unexpected character {}",
                    ch
                )));
            }
        }
    }

    tokens.push(Token::End);

    Ok(tokens)
}


fn get_line() -> String {
    print!("> ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_s) => {}
        Err(_e) => {}
    }
    input.trim().to_string()
}




fn eval(code: String) -> Result<(), Box<dyn Error>> {
    let tokens = lex(code)?;
    let mut token_iter = tokens.iter().peekable();
    let mut parser = Parser::new(&mut token_iter);
    let result = parser.parse();
    match result {
        Ok(ast) => println!("{}", ast.eval()),
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}

fn run_repl() -> Result<(), Box<dyn Error>> {
    loop {
        let line = get_line();
        if line == "quit" || line == "exit" || line == "q" {
            break;
        }
        if let Err(e) = eval(line) {
            println!("Error: {}", e);
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    run_repl()
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}
