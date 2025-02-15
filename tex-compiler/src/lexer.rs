// TODO: Support UTF-8 non-ASCII
//       Keep a buffer of the char indices with .char_indices()

// TODO: make private what must be private

use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
};

use regex::Regex;

const SEPARATORS: &[&str] = &[
    "{", "}", "\\[", "\\]", "$", "\\&", "\\#", "\\\\", "\\{", "\\}", "\\;", "\\,", "\\ ", "\\!",
    "[", "]", "\\|", "=",
];

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Text,
    Command,
    Separator,
    Whitespace,
    Newline,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub position: usize,
    pub data: String,
}

impl Token {
    fn new(kind: TokenKind, line: usize, position: usize, data: String) -> Self {
        Self {
            kind,
            line,
            position,
            data,
        }
    }
}

pub struct Lexer {
    reader: BufReader<File>,
    buffer: String,
    char_indices: Vec<usize>,
    line: usize,
    token_start: usize,
    token_end: usize,
    re_command: Regex,
    re_text: Regex,
    re_whitespace: Regex,
}

impl Lexer {
    pub fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
            buffer: String::new(),
            char_indices: Vec::new(),
            line: 0,
            token_start: 0,
            token_end: 0,
            re_command: Regex::new(r"^\\\w+$").unwrap(),
            re_text: Regex::new(r"^[^{}\\\[\]$=\s][^{}\\\[\]$=\n]*$").unwrap(),
            re_whitespace: Regex::new(r"^[^\S\n]+$").unwrap(),
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>> {
        // when at end of line, read new line
        if self.token_start + 1 >= self.char_indices.len() {
            if !self.read_line()? {
                return Ok(None);
            }
        }

        // when encountering comment start, ignore rest of the line
        if self.selection() == "%" {
            if !self.read_line()? {
                return Ok(None);
            }
            return self.next_token();
        }

        loop {
            if self.token_end + 1 >= self.char_indices.len() {
                // TODO: remove + 1 ?
                break;
            }
            self.token_end += 1; // try increasing selection
            if !self.is_token(self.selection()) {
                self.token_end -= 1; // undo increasing selection
                break;
            }
        }

        // Construct token from string view
        let token = self.to_token(self.selection());

        // Update `token_start` and `token_end`
        self.token_start = self.token_end;
        self.token_end = self.token_start + 1;

        Ok(Some(token))
    }

    fn read_line(&mut self) -> Result<bool> {
        self.buffer.clear();
        self.char_indices.clear();

        // length of line is zero means no more lines
        if self.reader.read_line(&mut self.buffer)? == 0 {
            return Ok(false);
        }

        self.char_indices
            .extend(self.buffer.char_indices().map(|(i, _)| i));
        self.char_indices.push(self.buffer.len());

        self.line += 1;
        self.token_start = 0;
        self.token_end = 1;

        Ok(true)
    }

    fn selection(&self) -> &str {
        let i = self.char_indices[self.token_start];
        let j = self.char_indices[self.token_end];
        &self.buffer[i..j]
    }

    fn is_token(&self, expr: &str) -> bool {
        if SEPARATORS.contains(&expr) {
            return true;
        }
        if self.re_command.is_match(&expr) {
            return true;
        }
        if self.re_text.is_match(&expr) {
            return true;
        }
        if self.re_whitespace.is_match(&expr) {
            return true;
        }
        if expr == "\n" {
            return true;
        }
        return false;
    }

    fn to_token(&self, expr: &str) -> Token {
        // panic if invalid, because is_token should be checked before!
        if SEPARATORS.contains(&expr) {
            return self.create_token(TokenKind::Separator, expr.to_owned());
        }
        if expr == "\n" {
            return self.create_token(TokenKind::Newline, expr.to_owned());
        }
        if self.re_whitespace.is_match(&expr) {
            return self.create_token(TokenKind::Whitespace, expr.to_owned());
        }
        if self.re_command.is_match(&expr) {
            return self.create_token(TokenKind::Command, expr.to_owned());
        }

        self.create_token(TokenKind::Text, expr.to_owned())
    }

    fn create_token(&self, kind: TokenKind, data: String) -> Token {
        let position = self.char_indices[self.token_start];
        Token::new(kind, self.line, position, data)
    }
}
