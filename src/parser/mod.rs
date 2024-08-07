use std::ops::Range;

use crate::lexer::token::{Token, TokenKind};

pub mod _type;
pub mod error;
pub mod name;
pub mod stmt;
pub mod var;

pub struct Input {
    stream: Vec<Token>,
    pos: usize,
    prev_pos: Range<usize>,
    prev_row: usize,
}

impl Input {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: tokens,
            pos: 0,
            prev_pos: 0..0,
            prev_row: 0,
        }
    }

    pub fn eat(&mut self) -> Option<&Token> {
        let result = if self.pos > self.stream.len() - 1 {
            None
        } else {
            let tok = &self.stream[self.pos];

            self.prev_pos = tok.pos();
            self.prev_row = tok.row_col().0;

            Some(tok)
        };

        self.pos += 1;

        result
    }

    pub fn eat_x(&mut self, x: usize) -> Option<&Token> {
        if self.pos + x + 1 >= self.stream.len() {
            None
        } else {
            self.pos += x;
            Some(&self.stream[self.pos])
        }
    }

    pub fn peek_x(&self, x: usize) -> Option<&Token> {
        if self.pos + x > self.stream.len() - 1 {
            None
        } else {
            Some(&self.stream[self.pos + x])
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.pos > self.stream.len() - 1 {
            None
        } else {
            Some(&self.stream[self.pos])
        }
    }

    pub fn peek_vec(&self, x: usize) -> Option<Vec<&Token>> {
        let mut results: Vec<&Token> = vec![];

        for i in 0..x {
            let tok = self.peek_x(i)?;
            results.push(tok);
        }

        Some(results)
    }

    pub fn match_pattern(&mut self, pattern: Vec<TokenKind>) -> bool {
        let found = match self.peek_vec(pattern.len()) {
            Some(tks) => tks,
            None => {
                return false;
            }
        };

        let mut i = 0;
        loop {
            if i >= pattern.len() - 1 {
                break;
            }

            if found[i].kind() != pattern[i] {
                return false;
            }

            i += 1;
        }

        true
    }

    // TODO: this eats the pattern, I am a bit dumb :(
    // pub fn skip_to_pattern(&mut self, patterns: Vec<Vec<TokenKind>>) -> Option<&Token> {
    //     let mut matched_index = 0;

    //     for (i, pattern) in patterns.into_iter().enumerate() {
    //         if self.match_pattern(pattern) {
    //             matched_index = i;
    //             break;
    //         }
    //     }

    //     self.eat_x(matched_index)
    // }
}
