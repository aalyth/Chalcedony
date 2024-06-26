use crate::error::{color, span::Position, Colors};

use super::Spanning;

/* the length of the shortened code annotation `...` */
const ELLIPSIS_LEN: usize = 3;

/// A spanner implementation, which builds code snippets, storing the whole
/// source code into the memory. This at first glance naive approach is
/// sufficiently efficient in terms of memory and fully optimized in terms of
/// lookup time.
pub struct InlineSpanner {
    src: Vec<String>,
    filename: Option<String>,
}

impl Spanning for InlineSpanner {
    fn context(&self, start: &Position, end: &Position) -> String {
        let mut result = String::new();

        let (context, ctx_offset) = self.context_span(start, end);
        result.push_str(&context);
        result.push('\n');

        let ln_len = std::cmp::max(end.ln.to_string().len(), 4);
        for _ in 0..ln_len {
            result.push(' ');
        }
        result.push_str(&color(Colors::Blue, "| "));

        for _ in 0..ctx_offset - (ln_len + 2) {
            result.push(' ');
        }

        /* color offset are the chars, used to represent the colors */
        const COLOR_OFFSET: usize = 5 + 4 + 5 + 4;
        let max_len = context.chars().count() - ctx_offset - COLOR_OFFSET;

        let el_len = 1 + end.col - start.col;
        let dist = std::cmp::min(el_len, max_len);

        for _ in 0..dist {
            result.push_str(&color(Colors::Cyan, "^"));
        }
        result.push('\n');

        result
    }

    fn filename(&self) -> Option<String> {
        self.filename.clone()
    }
}

impl InlineSpanner {
    pub fn new(src_code: &str, filename: Option<String>) -> InlineSpanner {
        let mut result = Vec::<String>::new();
        result.push("".to_string());
        for i in src_code.chars() {
            let end_pos = result.len() - 1;
            match i {
                '\n' => result.push("".to_string()),
                _ => result[end_pos].push(i),
            }
        }
        InlineSpanner {
            src: result,
            filename,
        }
    }

    // Returns the context string and the relative index in the result string
    // of the start position.
    fn context_span(&self, start_: &Position, end_: &Position) -> (String, usize) {
        if start_.ln == 0 || start_.col == 0 {
            panic!("Error: span: context_span: invalid start position.\n");
        }
        if end_.ln == 0 || end_.col == 0 {
            panic!("Error: span: context_span: invalid start position.\n");
        }

        if start_.ln == end_.ln && start_.col == end_.col {
            return self.context_substr(start_, 0);
        }

        let start = Position::new(start_.ln - 1, start_.col - 1);
        let end = Position::new(end_.ln - 1, end_.col - 1);

        if start.ln > end.ln || (start.ln == end.ln && start.col > end.col) {
            panic!("Error: span: context_span: end position  preceeds start position.\n");
        }

        if start.ln > self.src.len() || start.col > self.src[start.ln].len() {
            panic!("Error: span: context_span: start position out of bounds.\n");
        }
        if end.ln > self.src.len() || end.col > self.src[end.ln].len() {
            panic!("Error: span: context_span: end position out of bounds.\n");
        }

        if start.ln == end.ln && end.col - start.col < 70 {
            return self.context_substr(start_, end.col - start.col);
        }

        if start.ln == end.ln {
            let mut result = "".to_string();

            let end_ln_str = end_.ln.to_string();
            let ln_len = std::cmp::max(end_ln_str.len(), 4);
            for _ in 0..ln_len - end_ln_str.len() {
                result.push(' ');
            }
            let curr_line = &self.src[start.ln];
            result.push_str(&color(Colors::Blue, &start.ln.to_string()));
            result.push_str(&color(Colors::Blue, "| "));

            let mut res_pos: usize = ln_len + start_.col;

            if start.col > 35 {
                result.push_str("...");
                result.push_str(&curr_line[start.col - 15..start.col + 15]);
                res_pos = ln_len + ELLIPSIS_LEN + 15 + 1;
            } else {
                result.push_str(&curr_line[..start.col + 15]);
            }

            result.push_str("...");
            result.push_str(&curr_line[end.col - 15..end.col]);
            if curr_line.len() - end.col > 15 {
                result.push_str(&curr_line[end.col..end.col + 15]);
                result.push_str("...");
            } else {
                result.push_str(&curr_line[end.col..]);
            }
            return (result, res_pos);
        }

        let mut result = "".to_string();
        let res = self.context_pos(start_);

        result.push_str(&res.0);
        result.push('\n');

        if end.ln - start.ln > 1 {
            let ln_len = std::cmp::max(end_.ln.to_string().len(), 4);
            for _ in 0..ln_len - 3 {
                result.push(' ');
            }
            result.push_str(&color(Colors::Blue, "...| "));
            result.push_str("...\n");
        }

        result.push_str(&self.context_pos(end_).0);

        (result, res.1)
    }

    // Returns the formatted string, containing the content around the given
    // position and the given index of the position relative to the formatted
    // string.
    fn context_pos(&self, pos_: &Position) -> (String, usize) {
        self.context_substr(pos_, 0)
    }

    // Similar to `context_pos()`, but takes the length of the substring, around
    // which the context wraps around. Returns the begining of the substring
    // relative to the context output.
    fn context_substr(&self, pos_: &Position, ctx_len: usize) -> (String, usize) {
        if pos_.ln == 0 || pos_.col == 0 {
            panic!("Error: InlineSpanner::context_substr(): invalid position.");
        }

        let mut pos: Position = Position::new(pos_.ln - 1, pos_.col - 1);
        if pos.ln > self.src.len() {
            panic!("Error: InlineSpanner::context_substr(): position out of bounds.");
        }
        if pos.col > self.src[pos.ln].len() {
            panic!("Error: InlineSpanner::context_substr(): position out of bounds.");
        }

        let curr_line = &self.src[pos.ln];
        let pos_len = pos_.ln.to_string().len();
        let ln_len = std::cmp::max(pos_len, 4); //so the '|' is at least one tab inside

        let mut result = "".to_string();
        for _ in 0..(ln_len - pos_len) {
            result.push(' ');
        }
        result.push_str(&color(Colors::Blue, &pos_.ln.to_string()));
        result.push_str(&color(Colors::Blue, "| "));

        let mut res_pos: usize = ln_len + pos_.col;

        if pos.col > 35 {
            let left_bound = pos.col - 35;
            let right_bound = pos.col + ctx_len;

            result.push_str("...");
            let tmp: String = curr_line
                .chars()
                .take(right_bound)
                .skip(left_bound)
                .collect();
            result.push_str(&tmp);
            res_pos = ln_len + ELLIPSIS_LEN + 35 + 1;
        } else {
            let tmp: String = curr_line.chars().take(pos.col + ctx_len).collect();
            result.push_str(&tmp);
        }

        if ctx_len > 70 {
            result.push_str("...");
            pos.col += ctx_len - 70;
        }

        if curr_line.chars().count() - (pos.col + ctx_len) > 35 {
            // same as curr_line[pos.col + len .. pos.col + len + 24]
            // but works with UTF-8
            let tmp: String = curr_line
                .chars()
                .take(pos.col + ctx_len + 34)
                .skip(pos.col + ctx_len)
                .collect();
            result.push_str(&tmp);
            result.push_str("...");
        } else {
            let tmp: String = curr_line.chars().skip(pos.col + ctx_len).collect();
            result.push_str(&tmp);
        }

        (result, res_pos)
    }
}
