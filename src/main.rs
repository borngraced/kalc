use anyhow::{Result, anyhow};
use std::{env::args, io::stdin, iter::Peekable, slice::Iter};

const VERSION: &str = "0.1.2";

#[derive(Debug, Clone)]
enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Number(f64),
    Eof,
}

fn tokenize<'a>(mut src: Peekable<Iter<'a, char>>) -> Result<Vec<Token>> {
    let Some(n) = src.peek() else {
        return Err(anyhow!("Invalid math expression"));
    };
    if !n.is_numeric() {
        return Err(anyhow!("Invalid math expression"));
    };

    let mut tokens = vec![];
    while let Some(n) = src.next() {
        match n {
            '-' => tokens.push(Token::Sub),
            '+' => tokens.push(Token::Add),
            'x' => tokens.push(Token::Mul),
            '/' => tokens.push(Token::Div),
            '0'..='9' => {
                let mut digits = String::from(*n);
                let mut has_decimal = false;
                while let Some(&&k) = src.peek() {
                    if !k.is_numeric() {
                        if k == '.' {
                            if has_decimal {
                                return Err(anyhow!("Invalid math expression"));
                            }

                            has_decimal = true;
                        } else {
                            break;
                        }
                    }
                    digits.push(k);
                    src.next();
                }
                tokens.push(Token::Number(digits.parse::<f64>()?));
            }
            ' ' | '\n' => continue,
            _ => return Err(anyhow!("Unrecognized character: {}", n)),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[derive(Debug, Clone)]
enum Op {
    Mul,
    Add,
    Sub,
    Div,
}

#[derive(Debug, Clone)]
enum ASTNode {
    Number(f64),
    BinaryOp {
        left: Box<ASTNode>,
        op: Op,
        right: Box<ASTNode>,
    },
}

impl ASTNode {
    fn eval(&self) -> f64 {
        match self {
            ASTNode::Number(n) => *n,
            ASTNode::BinaryOp { left, op, right } => {
                let left = left.eval();
                let right = right.eval();
                match op {
                    Op::Mul => left * right,
                    Op::Add => left + right,
                    Op::Sub => left - right,
                    Op::Div => left / right,
                }
            }
        }
    }
}

#[derive(Debug)]
struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Peekable<Iter<'a, Token>>) -> Self {
        Self { tokens }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek().copied()
    }

    fn advance(&mut self) -> Option<&Token> {
        self.tokens.next()
    }

    fn parse_program(&mut self) -> Result<Option<ASTNode>> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Option<ASTNode>> {
        let Some(mut expr) = self.parse_multiplicative()? else {
            return Ok(None);
        };

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Add => Op::Add,
                Token::Sub => Op::Sub,
                Token::Number(_) => return Err(anyhow!("Invalid math expression")),
                _ => break,
            };
            self.advance();

            if let Some(right) = self.parse_multiplicative()? {
                expr = ASTNode::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(Some(expr))
    }

    fn parse_multiplicative(&mut self) -> Result<Option<ASTNode>> {
        let Some(mut expr) = self.parse_primary_exp()? else {
            return Ok(None);
        };

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Mul => Op::Mul,
                Token::Div => Op::Div,
                Token::Number(_) => return Err(anyhow!("Invalid math expression")),
                _ => break,
            };
            self.advance();

            if let Some(right) = self.parse_primary_exp()? {
                expr = ASTNode::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(Some(expr))
    }

    fn parse_primary_exp(&mut self) -> Result<Option<ASTNode>> {
        let Some(Token::Number(n)) = self.peek() else {
            return Ok(None);
        };
        let n = *n;
        Ok(self.advance().map(|_| ASTNode::Number(n)))
    }
}

fn format_float(num: f64) -> String {
    if num.fract() == 0.0 {
        format!("{:.0}", num)
    } else {
        format!("{}", num)
    }
}

fn print_help() {
    println!("kalc-cli");
    println!();

    println!("USAGE:");
    println!("  kalc [OPTIONS] [EXPRESSION]");
    println!();

    println!("OPTIONS:");
    println!("  -h, --help     Display this help message");
    println!("  -v, --version  Display version information");
    println!();

    println!("EXPRESSION SYNTAX:");
    println!("  Basic arithmetic: +, -, x, /");
    println!("  Numbers can be integers or decimals");
    println!();

    println!("EXAMPLES:");
    println!("  kalc 2 + 3 * 4");
    println!("  kalc 5 + 3 / 2");
    println!("  kalc 3.14 * 2.5");
    println!();

    println!("NOTES:");
    println!("  - If no expression is provided, kalcwill read from stdin");
    println!();

    println!("VERSION:");
    println!("  kalc-cli {VERSION}");
}

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();

    if args.len() > 1 {
        if args[1] == "-h" || args[1] == "--help" {
            print_help();
            return Ok(());
        } else if args[1] == "-v" || args[1] == "--version" {
            println!("kalc {VERSION}");
            return Ok(());
        }
    }

    let expr = if args.len() <= 1 {
        println!("kalc {VERSION}");
        println!("Enter an expression (or type 'help' for instructions):");

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        if input.trim() == "help" {
            print_help();
            return Ok(());
        } else if input.trim().is_empty() {
            return Err(anyhow!("No expression provided"));
        }

        input
    } else {
        args[1..].join(" ")
    };

    let chars = expr.chars().collect::<Vec<char>>();
    let tokens = tokenize(chars.iter().peekable())?;
    let mut ast = Parser::new(tokens.iter().peekable());
    let ast = ast
        .parse_program()?
        .ok_or(anyhow!("unable to parse expression"))?;
    let result = format_float(ast.eval());

    println!("{result}");

    Ok(())
}
