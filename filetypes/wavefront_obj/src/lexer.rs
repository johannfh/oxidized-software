use std::io::{BufRead, BufReader, Error as IoError, Read, Result as IoResult};

use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum ObjToken {
    // --- Keywords ---
    /// v (vertex)
    V,
    /// vt (texture coordinate)
    Vt,
    /// vn (vertex normal)   
    Vn,
    /// f (face)
    F,
    /// o (object name)
    O,
    /// g (group name)
    G,
    /// s (smoothing group)
    S,
    /// mtllib (material library)
    Mtllib,
    /// usemtl (use material)
    Usemtl,

    // --- Datatypes ---
    /// 32-bit float primitive type
    Float(f32),
    /// 32-bit int primitive type
    Integer(i32),
    /// For names like material names, object names, group names
    Identifier(String),
    /// For file paths like .mtl files
    Path(String),

    // --- Punctuation ---
    Slash,
    Hash,

    // --- Special ---
    /// End of file
    Eof,
    Newline,
    Backslash,
    Whitespace,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error(
        "Unexpected token at physical line {line}, column {column}: expected {expected}, found {found:?}"
    )]
    UnexpectedToken {
        expected: &'static str,
        found: ObjToken,
        line: usize,
        column: usize,
    },
}

pub struct Lexer<R>
where
    R: Read,
{
    reader: BufReader<R>,
    current_line: String,
    line_idx: usize,
    physical_line_number: usize,
    column_number: usize,
    is_eof: bool,
}

impl<R> Lexer<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Lexer<R> {
        Self {
            reader: BufReader::new(reader),
            current_line: String::new(),
            line_idx: 0,
            physical_line_number: 0,
            column_number: 0,
            is_eof: false,
        }
    }

    fn read_next_logical_line(&mut self) -> IoResult<bool> {
        self.current_line.clear();

        let mut temp_line = String::new();
        let mut read_any_line = false;

        loop {
            temp_line.clear();
            let bytes_read = self.reader.read_line(&mut temp_line)?;

            if bytes_read == 0 {
                self.is_eof = true;
                return Ok(read_any_line)
            }

            self.physical_line_number += 1;
            read_any_line = true;

            let trimmed_temp_line = temp_line.trim_end_matches(['\n', '\r']);
            if trimmed_temp_line.ends_with('\\') {
                // push up until the '\' character
                self.current_line.push_str(&trimmed_temp_line[0..trimmed_temp_line.len() - 1]);
            } else {
                // push whole line
                self.current_line.push_str(trimmed_temp_line);
                break;
            }
        }

        Ok(true)
    }

    fn peek_char(&self) -> Option<char> {
        if self.is_eof {
            return None;
        }
        self.current_line.chars().nth(self.line_idx)
    }

    fn consume_char(&mut self) -> Option<char> {
        if self.is_eof {
            return None;
        }

        let current_char_opt = self.current_line.chars().nth(self.line_idx);
        if current_char_opt.is_some() {
            self.line_idx += 1;
            self.column_number += 1;
        }

        current_char_opt
    }

    fn read_digits(&mut self) -> IoResult<String> {
        let mut s = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                s.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        Ok(s)
    }

    fn read_identifier(&mut self) -> IoResult<String> {
        let mut s = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' {
                s.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        Ok(s)
    }

    fn next_token(&mut self) -> IoResult<ObjToken> {
        loop {
            if self.peek_char().is_none() && !self.is_eof {
                if !self.read_next_logical_line()? {
                    return Ok(ObjToken::Eof);
                }
            }

            let current_char = if let Some(c) = self.peek_char() {
                c
            } else {
                return Ok(ObjToken::Eof);
            };


            todo!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;

    #[test]
    fn test_lex_vertex() {
        let obj_content = r#"
            v 5.0 5.0 5.0
        "#;

        let lexer = Lexer::new(obj_content.as_bytes());
    }
}
