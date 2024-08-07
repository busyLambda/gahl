use self::token::Span;

pub mod token;
pub mod tokenizer;

const EOF: char = '\0';

pub struct Lexer {
    chars: Vec<char>,
    cursor: usize,
    pos_within_tok: usize,
    pos_last_tok: usize,
    tok_str: String,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            chars: input.chars().collect(),
            cursor: 0,
            pos_within_tok: 0,
            pos_last_tok: 0,
            tok_str: String::new(),
            row: 0,
            col: 0,
        }
    }

    fn inc_row(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    fn is_eof(&self) -> bool {
        self.chars.len() == 0 || self.cursor > self.chars.len() - 1
    }

    fn peek(&self) -> Option<char> {
        if self.is_eof() {
            return None;
        }

        Some(self.chars[self.cursor])
    }

    fn eat(&mut self) -> char {
        if self.is_eof() {
            return EOF;
        }

        let c = self.chars[self.cursor];

        self.col += 1;
        self.cursor += 1;
        self.pos_within_tok += 1;

        self.tok_str.push(c);
        c
    }

    fn eat_while(&mut self, pred: fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if pred(c) {
                self.eat();
            } else {
                break;
            }
        }
    }

    fn reset_pos_within_tok(&mut self) {
        self.pos_last_tok = self.cursor;
        self.pos_within_tok = 0;
        self.tok_str.clear();
    }

    fn span(&self) -> Span {
        let start = self.pos_last_tok;
        let end = self.cursor;

        let literal = self.tok_str.clone();

        Span::new(start, end, literal)
    }
}
