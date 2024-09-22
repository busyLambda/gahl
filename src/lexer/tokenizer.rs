use super::{
    token::{Span, Token, TokenKind},
    Lexer,
};

impl Lexer {
    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            let token = self.next();
            tokens.push(token);
        }

        tokens
            .into_iter()
            .filter(|t| t.kind() != TokenKind::Whitespace)
            .collect::<Vec<Token>>()
    }

    fn next(&mut self) -> Token {
        if self.is_eof() {
            return Token::new(
                TokenKind::EOF,
                Span::new(0, 0, String::new()),
                self.row,
                self.col,
            );
        }

        let c = self.eat();

        let kind = match c {
            c if c.is_alphabetic() => self.ident_or_kw_or_type(),
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::ClosedParen,
            '{' => TokenKind::OpenCurly,
            '}' => TokenKind::ClosedCurly,
            ':' => self.col_or_coleq(),
            '=' => self.eq_or_eqeq(),
            '+' => TokenKind::Add,
            '-' => self.min_or_right_arrow(),
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '%' => TokenKind::Mod,
            '^' => TokenKind::Caret,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::ClosedBracket,
            '@' => TokenKind::At,
            '"' => {
                self.eat();

                self.string()
            }
            ';' => {
                self.eat();

                self.doc_comment()
            }
            '\n' => {
                self.inc_row();
                TokenKind::Whitespace
            }
            c if c.is_numeric() => self.integer(),
            c if c.is_whitespace() => self.whitespace(),
            _ => TokenKind::Unknown,
        };

        let span = self.span();

        let token = Token::new(kind, span, self.row, self.col);
        self.reset_pos_within_tok();

        token
    }

    fn doc_comment(&mut self) -> TokenKind {
        if let Some('\n') = self.peek() {
            self.inc_row();
        }

        self.eat_while(|c| c != ';');

        self.eat();

        TokenKind::DocComment
    }

    // TODO: Implement escapes and such.
    fn string(&mut self) -> TokenKind {
        if let Some('\n') = self.peek() {
            self.inc_row();
        }

        self.eat_while(|c| c != '"');

        self.eat();

        TokenKind::String
    }

    fn ident_or_kw_or_type(&mut self) -> TokenKind {
        self.eat_while(is_ident_cont);

        match self.tok_str.as_str() {
            // Keywords
            "fn" => TokenKind::KwFn,
            "if" => TokenKind::KwIf,
            "match" => TokenKind::KwMatch,
            "import" => TokenKind::KwImport,
            "struct" => TokenKind::KwStruct,
            "enum" => TokenKind::KwEnum,
            // Types
            "void" => TokenKind::Tvoid,
            // boolean
            "bool" => TokenKind::Tbool,
            // int
            "i8" => TokenKind::Ti8,
            "i16" => TokenKind::Ti16,
            "i32" => TokenKind::Ti32,
            "i64" => TokenKind::Ti64,
            "i128" => TokenKind::Ti128,
            // uint
            "u8" => TokenKind::Tu8,
            "u16" => TokenKind::Tu16,
            "u32" => TokenKind::Tu32,
            "u64" => TokenKind::Tu64,
            "u128" => TokenKind::Tu128,
            // float
            "f32" => TokenKind::Tf32,
            "f64" => TokenKind::Tf64,
            // char, string
            "char" => TokenKind::Tchar,
            "string" => TokenKind::Tstring,
            // Else
            "not" => TokenKind::Not,
            _ => TokenKind::Identifier,
        }
    }

    fn col_or_coleq(&mut self) -> TokenKind {
        if let Some(c) = self.peek() {
            if c == '=' {
                self.eat();
                return TokenKind::Coleq;
            }
        }
        TokenKind::Column
    }

    fn eq_or_eqeq(&mut self) -> TokenKind {
        if let Some(c) = self.peek() {
            if c == '=' {
                self.eat();
                return TokenKind::EqEq;
            }
        }
        TokenKind::Eq
    }

    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn integer(&mut self) -> TokenKind {
        self.eat_while(is_integer_cont);
        TokenKind::Integer
    }

    fn min_or_right_arrow(&mut self) -> TokenKind {
        if let Some(c) = self.peek() {
            if c == '>' {
                self.eat();
                return TokenKind::RightArrow;
            }
        }
        TokenKind::Min
    }
}

fn is_ident_cont(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

fn is_integer_cont(c: char) -> bool {
    c.is_numeric()
}
