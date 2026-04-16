use crate::r#type::{ColumnDef, DataType, Statement, Value, WhereClause};

// ─── Keywords ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Keyword {
    Create,
    Table,
    Insert,
    Into,
    Values,
    Select,
    From,
    Where,
    Int,
    Text,
}

// ─── Tokens ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Keyword(Keyword),
    Identifier(String),
    IntLiteral(i64),
    StringLiteral(String),
    Comma,
    LParen,
    RParen,
    Equals,
    Asterisk,
    Semicolon,
}

// ─── Tokenizer ────────────────────────────────────────────────────────────────

fn word_to_token(word: &str) -> Token {
    match word {
        "CREATE" => Token::Keyword(Keyword::Create),
        "TABLE" => Token::Keyword(Keyword::Table),
        "INSERT" => Token::Keyword(Keyword::Insert),
        "INTO" => Token::Keyword(Keyword::Into),
        "VALUES" => Token::Keyword(Keyword::Values),
        "SELECT" => Token::Keyword(Keyword::Select),
        "FROM" => Token::Keyword(Keyword::From),
        "WHERE" => Token::Keyword(Keyword::Where),
        "INT" => Token::Keyword(Keyword::Int),
        "TEXT" => Token::Keyword(Keyword::Text),
        _ => Token::Identifier(word.to_string()),
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut word = String::new();
    let mut number = String::new();

    // index-based loop so we can jump ahead when reading string literals
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_alphabetic() || c == '_' {
            // flush any pending number first
            if !number.is_empty() {
                let n = number.parse::<i64>().map_err(|e| e.to_string())?;
                tokens.push(Token::IntLiteral(n));
                number.clear();
            }
            word.push(c.to_ascii_uppercase());
            i += 1;
        } else if c.is_ascii_digit() {
            // flush any pending word first
            if !word.is_empty() {
                tokens.push(word_to_token(&word));
                word.clear();
            }
            number.push(c);
            i += 1;
        } else if c == '\'' {
            // flush pending word/number first
            if !word.is_empty() {
                tokens.push(word_to_token(&word));
                word.clear();
            }
            if !number.is_empty() {
                let n = number.parse::<i64>().map_err(|e| e.to_string())?;
                tokens.push(Token::IntLiteral(n));
                number.clear();
            }
            // collect everything between the two ' marks
            i += 1; // skip opening '
            let mut s = String::new();
            while i < chars.len() && chars[i] != '\'' {
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string literal".to_string());
            }
            i += 1; // skip closing '
            tokens.push(Token::StringLiteral(s));
        } else {
            // flush pending word/number before handling symbol or space
            if !word.is_empty() {
                tokens.push(word_to_token(&word));
                word.clear();
            }
            if !number.is_empty() {
                let n = number.parse::<i64>().map_err(|e| e.to_string())?;
                tokens.push(Token::IntLiteral(n));
                number.clear();
            }

            if c.is_whitespace() {
                i += 1;
                continue;
            }

            let tok = match c {
                ',' => Token::Comma,
                '(' => Token::LParen,
                ')' => Token::RParen,
                '=' => Token::Equals,
                '*' => Token::Asterisk,
                ';' => Token::Semicolon,
                _ => return Err(format!("Unexpected character: '{}'", c)),
            };
            tokens.push(tok);
            i += 1;
        }
    }

    // flush anything left over at end of input
    if !word.is_empty() {
        tokens.push(word_to_token(&word));
    }
    if !number.is_empty() {
        let n = number.parse::<i64>().map_err(|e| e.to_string())?;
        tokens.push(Token::IntLiteral(n));
    }

    Ok(tokens)
}

// ─── Parser ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn consume(&mut self) -> Token {
        let tok = self.tokens[self.position].clone();
        self.position += 1;
        tok
    }

    fn expect_keyword(&mut self, key: Keyword) -> Result<(), String> {
        match self.peek() {
            Some(Token::Keyword(k)) if *k == key => {
                self.consume();
                Ok(())
            }
            Some(t) => Err(format!("Expected {:?}, found {:?}", key, t)),
            None => Err(format!("Expected {:?}, found end of input", key)),
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        match self.peek() {
            Some(Token::Identifier(_)) => {
                if let Token::Identifier(name) = self.consume() {
                    Ok(name)
                } else {
                    unreachable!()
                }
            }
            Some(t) => Err(format!("Expected identifier, found {:?}", t)),
            None => Err("Expected identifier, found end of input".to_string()),
        }
    }

    fn parse_data_type(&mut self) -> Result<DataType, String> {
        match self.peek() {
            Some(Token::Keyword(Keyword::Int)) => {
                self.consume();
                Ok(DataType::Integer)
            }
            Some(Token::Keyword(Keyword::Text)) => {
                self.consume();
                Ok(DataType::Text)
            }
            Some(t) => Err(format!("Expected INT or TEXT, found {:?}", t)),
            None => Err("Expected data type, found end of input".to_string()),
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some(Token::IntLiteral(_)) => {
                if let Token::IntLiteral(n) = self.consume() {
                    Ok(Value::Integer(n))
                } else {
                    unreachable!()
                }
            }
            Some(Token::StringLiteral(_)) => {
                if let Token::StringLiteral(s) = self.consume() {
                    Ok(Value::Text(s))
                } else {
                    unreachable!()
                }
            }
            Some(t) => Err(format!("Expected a value, found {:?}", t)),
            None => Err("Expected a value, found end of input".to_string()),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        let mut word1 = self.expect_keyword(Keyword::Table)?;
        let mut table_name = self.expect_identifier()?;
        let paran = self.consume(); // consume 'TABLE'
        println!("paran {:?} ", paran);

        let mut column = Vec::new();
        loop {
            let column_name = self.expect_identifier()?;
            let data_type = self.parse_data_type()?;

            column.push(ColumnDef {
                name: column_name,
                data_type,
            });

            let mut key = self.peek();
            match key {
                Some(Token::Comma) => {
                    self.consume();
                    continue;
                }
                Some(Token::RParen) => {
                    self.consume();
                    break;
                }
                Some(t) => return Err(format!("Expected ',' or ')', found {:?}", t)),
                None => return Err("Expected ',' or ')', found end of input".to_string()),
            }
        }

        return Ok(Statement::CreateTable {
            table_name,
            columns: column,
        });
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        let mut key = self.expect_keyword(Keyword::Into)?;
        let table_name = self.expect_identifier()?;
        let key2 = self.expect_keyword(Keyword::Values)?;
        let leftparan = self.consume();
        match leftparan {
            Token::LParen => {} // OK
            _ => return Err("Left_paranthesis_not_found".to_string()),
        }

        let mut values = Vec::new();

        loop {
            let val = self.parse_value()?;
            values.push(val);
            let token = self.peek();

            match token {
                Some(Token::Comma) => {
                    self.consume();
                    continue;
                }
                Some(Token::RParen) => {
                    self.consume();
                    break;
                }
                Some(t) => return Err(format!("Expected ',' or ')', found {:?}", t)),
                None => return Err("Expected ',' or ')', found end of input".to_string()),
            }
        }
        return Ok(Statement::Insert {
            table_name,
            values: values,
        });
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        // assume SELECT already consumed before calling this

        // handle '*' (you can expand later)
        self.consume(); // consume '*'

        // expect FROM
        self.expect_keyword(Keyword::From)?;

        // table name
        let table_name = self.expect_identifier()?;

        // check if WHERE exists (optional)
        let where_clause = if let Some(Token::Keyword(Keyword::Where)) = self.peek() {
            self.consume(); // consume WHERE

            let column = self.expect_identifier()?;

            // expect '='
            match self.consume() {
                Token::Equals => {}
                t => return Err(format!("Expected '=', found {:?}", t)),
            }

            let value = self.parse_value()?;

            Some(WhereClause { column, value })
        } else {
            None
        };

        Ok(Statement::Select {
            table_name,
            where_clause,
        })
    }
}

// ─── Public API (Part E) — YOU WRITE THIS ─────────────────────────────────────

pub fn parse(input: &str) -> Result<Statement, String> {
    let cleaned = input.trim_end_matches(";");
    let tokens = tokenize(cleaned)?;
    let mut parser = Parser::new(tokens);
    match parser.consume() {
        Token::Keyword(Keyword::Create) => {
            parser.parse_create_table()
        }
        Token::Keyword(Keyword::Insert) => {
            parser.parse_insert()
        }
        Token::Keyword(Keyword::Select) => {
            parser.parse_select()
        }
        _ => {
            Err(format!("Unknow statment is passed {:?}",parser))
        }
    }
}
