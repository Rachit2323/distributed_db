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
    Delete,
    Update,
    Set,
    Primary,
    Key,
    Index,
    On,
    Drop,
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
        "DELETE" => Token::Keyword(Keyword::Delete),
        "UPDATE" => Token::Keyword(Keyword::Update),
        "SET" => Token::Keyword(Keyword::Set),
        "PRIMARY" => Token::Keyword(Keyword::Primary),
        "KEY" => Token::Keyword(Keyword::Key),
        "INDEX" => Token::Keyword(Keyword::Index),
        "ON" => Token::Keyword(Keyword::On),
        "DROP" => Token::Keyword(Keyword::Drop),
        _ => Token::Identifier(word.to_string()),
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut word = String::new();
    let mut number = String::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_alphabetic() || c == '_' {
            if !number.is_empty() {
                let n = number.parse::<i64>().map_err(|e| e.to_string())?;
                tokens.push(Token::IntLiteral(n));
                number.clear();
            }
            word.push(c.to_ascii_uppercase());
            i += 1;
        } else if c.is_ascii_digit() {
            if !word.is_empty() {
                tokens.push(word_to_token(&word));
                word.clear();
            }
            number.push(c);
            i += 1;
        } else if c == '\'' {
            if !word.is_empty() {
                tokens.push(word_to_token(&word));
                word.clear();
            }
            if !number.is_empty() {
                let n = number.parse::<i64>().map_err(|e| e.to_string())?;
                tokens.push(Token::IntLiteral(n));
                number.clear();
            }
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '\'' {
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string literal".to_string());
            }
            i += 1;
            tokens.push(Token::StringLiteral(s));
        } else {
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
        self.expect_keyword(Keyword::Table)?;
        let table_name = self.expect_identifier()?;
        self.consume(); // consume '('

        let mut columns = Vec::new();
        let mut primary_key: Option<String> = None;

        loop {
            let column_name = self.expect_identifier()?;
            let data_type = self.parse_data_type()?;

            if let Some(Token::Keyword(Keyword::Primary)) = self.peek() {
                self.consume();
                self.expect_keyword(Keyword::Key)?; // consume KEY
                primary_key = Some(column_name.clone());
            }

            columns.push(ColumnDef {
                name: column_name,
                data_type,
            });

            match self.peek() {
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

        Ok(Statement::CreateTable {
            table_name,
            columns,
            primary_key,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect_keyword(Keyword::Into)?;
        let table_name = self.expect_identifier()?;
        self.expect_keyword(Keyword::Values)?;

        match self.consume() {
            Token::LParen => {}
            _ => return Err("Expected '('".to_string()),
        }

        let mut values = Vec::new();
        loop {
            values.push(self.parse_value()?);
            match self.peek() {
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

        Ok(Statement::Insert { table_name, values })
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.consume(); // consume '*'
        self.expect_keyword(Keyword::From)?;
        let table_name = self.expect_identifier()?;

        let where_clause = if let Some(Token::Keyword(Keyword::Where)) = self.peek() {
            self.consume();
            let column = self.expect_identifier()?;
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

    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.expect_keyword(Keyword::From)?;
        let table_name = self.expect_identifier()?;
        self.expect_keyword(Keyword::Where)?;
        let column_name = self.expect_identifier()?;

        match self.consume() {
            Token::Equals => {}
            t => return Err(format!("Expected '=', found {:?}", t)),
        }

        let val = self.parse_value()?;
        Ok(Statement::Delete {
            table_name,
            where_clause: WhereClause {
                column: column_name,
                value: val,
            },
        })
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        let table_name = self.expect_identifier()?;

        // SET column = new_value
        self.expect_keyword(Keyword::Set)?;
        let set_column = self.expect_identifier()?;
        match self.consume() {
            Token::Equals => {}
            t => return Err(format!("Expected '=', found {:?}", t)),
        }
        let set_value = self.parse_value()?;

        // WHERE column = match_value
        self.expect_keyword(Keyword::Where)?;
        let where_column = self.expect_identifier()?;
        match self.consume() {
            Token::Equals => {}
            t => return Err(format!("Expected '=', found {:?}", t)),
        }
        let where_value = self.parse_value()?;

        Ok(Statement::Update {
            table_name,
            column: set_column,
            value: set_value,
            where_clause: WhereClause {
                column: where_column,
                value: where_value,
            },
        })
    }

    fn parse_create_index(&mut self) -> Result<Statement, String> {
        self.expect_keyword(Keyword::Index)?;
        self.expect_keyword(Keyword::On)?;
        let table_name = self.expect_identifier()?;

        let lparen = self.consume();
        match lparen {
            Token::LParen => {}
            _ => return Err("Lparent not found ".to_string()),
        }

        let column = self.expect_identifier()?;

        let rparen = self.consume();
        match rparen {
            Token::RParen => {}
            _ => return Err("RParen not found ".to_string()),
        }

        return Ok(Statement::CreateIndex {
            table_name: table_name,
            column: column,
        });
    }

    fn parse_drop_index(&mut self) -> Result<Statement, String> {
        self.expect_keyword(Keyword::Index)?;
        self.expect_keyword(Keyword::On)?;
        let table_name = self.expect_identifier()?;

        let lparen = self.consume();
        match lparen {
            Token::LParen => {}
            _ => return Err("Lparent not found ".to_string()),
        }

        let column = self.expect_identifier()?;

        let rparen = self.consume();
        match rparen {
            Token::RParen => {}
            _ => return Err("RParen not found ".to_string()),
        }

        return Ok(Statement::DropIndex {
            table_name: table_name,
            column: column,
        });
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> Result<Statement, String> {
    let cleaned = input.trim_end_matches(';');
    let tokens = tokenize(cleaned)?;
    let mut parser = Parser::new(tokens);
    match parser.consume() {
        Token::Keyword(Keyword::Create) => match parser.consume() {
            Token::Keyword(Keyword::Table) => parser.parse_create_table(),
            Token::Keyword(Keyword::Index) => parser.parse_create_index(),
            _ => Err("Expected TABLE or INDEX".to_string()),
        },
        Token::Keyword(Keyword::Insert) => parser.parse_insert(),
        Token::Keyword(Keyword::Select) => parser.parse_select(),
        Token::Keyword(Keyword::Delete) => parser.parse_delete(),
        Token::Keyword(Keyword::Update) => parser.parse_update(),
        _ => Err(format!("Unknown statement, parser state: {:?}", parser)),
    }
}
