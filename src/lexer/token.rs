use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Span {
    start: usize,
    end: usize,
    literal: String,
}

impl Span {
    pub fn new(start: usize, end: usize, literal: String) -> Self {
        Self {
            start,
            end,
            literal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {

    // Types
    Tvoid,
    Tbool,
    Ti8,
    Ti16,
    Ti32,
    Ti64,
    Ti128,
    Tu8,
    Tu16,
    Tu32,
    Tu64,
    Tu128,
    Tf32,
    Tf64,
    Tchar,
    Tstring,

    // Keywords
    KwIf,
    KwMatch,
    KwFn,
    KwExtern,
    KwImport,
    KwStruct,
    KwEnum,

    // Funnies
    Dot,
    At,
    Comma,
    RightArrow,
    Column,
    Coleq,
    Eq,
    EqEq,
    OpenParen,
    ClosedParen,
    OpenCurly,
    ClosedCurly,
    OpenBracket,
    ClosedBracket,

    // Literals
    Integer,
    Identifier,
    String,

    // Ops
    Add,
    Min,
    Mul,
    Div,
    Caret,
    Mod,
    Not,

    // Special
    UnclosedComment,
    Comment,
    DocComment,
    Whitespace,
    NewLine, // For sync
    Unknown,
    EOF,
}

impl TokenKind {
    pub fn is_sync_token(&self) -> bool {
        matches!(
            self,
            Self::ClosedParen | Self::ClosedCurly | Self::ClosedBracket | Self::EOF
        )
    }

    pub fn is_begin_new_stmt(&self) -> bool {
        matches!(self, |Self::KwIf| Self::KwMatch | Self::KwFn)
    }

    pub fn is_stmt(&self) -> bool {
        matches!(
            self,
            Self::KwIf
                | Self::KwEnum
                | Self::KwFn
                | Self::KwMatch
                | Self::KwImport
                | Self::KwStruct
                | Self::Identifier
        )
    }

    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Self::Tvoid
                | Self::Tbool
                | Self::Ti8
                | Self::Ti16
                | Self::Ti32
                | Self::Ti64
                | Self::Ti128
                | Self::Tu8
                | Self::Tu16
                | Self::Tu32
                | Self::Tu64
                | Self::Tu128
                | Self::Tf32
                | Self::Tf64
                | Self::Tstring
                | Self::Tchar
                | Self::Identifier
        )
    }

    pub fn is_expr(&self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Identifier | Self::Min | Self::OpenParen | Self::KwFn | Self::KwExtern
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Min
                | Self::Mul
                | Self::Div
                | Self::Mod
                | Self::Not
                | Self::Eq
                | Self::EqEq
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    span: Span,
    row: usize,
    col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, row: usize, col: usize) -> Self {
        Self {
            kind,
            span,
            row,
            col,
        }
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn kind(&self) -> TokenKind {
        self.kind.clone()
    }

    pub fn literal(&self) -> String {
        self.span.literal.clone()
    }

    pub fn pos(&self) -> Range<usize> {
        self.span.start..self.span.end
    }
}
