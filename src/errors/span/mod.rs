pub mod pos;

use pos::Position;

pub struct Span {
    src: Vec<String>,
}

impl Span {
    pub fn new(src_code: &str) -> Span {
        let mut result = Vec::<String>::new();
        result.push("".to_string());
        for i in src_code.chars() {
            let end_pos = result.len() - 1;
            match i {
                '\n' => result.push("".to_string()),
                _    => result[end_pos].push(i),
            }
        }
        Span { src: result }
    }

    // returns the context string and the relative index in the result string of the start position
    pub fn context_span(&self, start_: &Position, end_: &Position) -> Result<(String, usize), &str> {
        if start_.ln == 0 || start_.col == 0 { return Err("Error: span: context_span: invalid start position.\n"); } 
        if end_.ln == 0 || end_.col == 0 { return Err("Error: span: context_span: invalid start position.\n"); } 

        let start = Position::new(start_.ln - 1, start_.col - 1);
        let end = Position::new(end_.ln - 1, end_.col - 1);

        if start.ln > end.ln || (start.ln == end.ln && start.col >= end.col) { 
            return Err("Error: span: context_span: end position matches or preceeds start position.\n");
        }

        if start.ln > self.src.len() || start.col > self.src[start.ln].len() {  
            return Err("Error: span: context_span: start position out of bounds.\n");
        }
        if end.ln > self.src.len() || end.col > self.src[end.ln].len() {  
            return Err("Error: span: context_span: end position out of bounds.\n");
        }
        
        if start.ln == end.ln && end.col - start.col < 40 { 
            return self.context_substr(start_, end.col - start.col); 

        } else if start.ln == end.ln { 
            let mut result = "".to_string();

            let end_ln_str = end_.ln.to_string();
            let ln_len = std::cmp::max(end_ln_str.len(), 4); 
            for _ in 0 .. ln_len - end_ln_str.len() { result.push_str(" "); }
            let curr_line = &self.src[start.ln];
            result.push_str(&start.ln.to_string());
            result.push_str("| ");
            let mut res_pos: usize = 0;

            if start.col > 15 {
                result.push_str("...");
                result.push_str(&curr_line[start.col-15 .. start.col + 15].to_string());
            } else {
                result.push_str(&curr_line[ .. start.col + 15].to_string());
            }
            res_pos = result.len() - 15;

            result.push_str("...");
            result.push_str(&curr_line[end.col-15 .. end.col].to_string());
            if curr_line.len() - end.col > 15 {
                result.push_str(&curr_line[end.col .. end.col + 15].to_string());
                result.push_str("...");
            } else {
                result.push_str(&curr_line[end.col .. ].to_string());
            }
            return Ok((result, res_pos));
        }
        
        let mut result = "".to_string();
        let mut res = self.context_pos(start_).unwrap();

        result.push_str(&res.0);
        result.push_str("\n");

        let ln_len = std::cmp::max(end_.ln.to_string().len(), 4); 
        for _ in 0 .. ln_len-3 { result.push_str(" "); }
        result.push_str("...| ...\n");

        result.push_str(&self.context_pos(end_).unwrap().0);

        Ok( (result, res.1) )
    }

    // returns a formatted string, containing the content around the given position
    //
    // if successful returns the formated string and the given index of the position 
    // relative to the formated string
    //
    // pos: (ln, col)
    pub fn context_pos(&self, pos_: &Position) -> Result<(String, usize), &str> {
        self.context_substr(pos_, 0)
    }

    // similar to context_pos(), but takes the length of the substring, to which the context wraps around
    //
    // returns the begining of the substring relative to the context output
    pub fn context_substr(&self, pos_: &Position, len: usize) -> Result<(String, usize), &str> {
        if pos_.ln == 0 || pos_.col == 0 { return Err("Error: span: context_substr: invalid position."); } 

        let pos: Position = Position::new(pos_.ln - 1, pos_.col - 1);
        if pos.ln > self.src.len()  { return Err("Error: span: context_substr: position out of bounds."); } 
        if pos.col > self.src[pos.ln].len() { return Err("Error: span: context_substr: position out of bounds."); } 

        let curr_line = &self.src[pos.ln];
        let pos_len = pos_.ln.to_string().len();
        let ln_len = std::cmp::max(pos_len, 4); //so the '|' is at least one tab
                                                                 //inside
        let mut result = "".to_string();
        for _ in 0 .. (ln_len - pos_len) { result.push_str(" "); }
        result.push_str(&pos_.ln.to_string());
        result.push_str("| ");
        let mut res_pos: usize = 0;

        if pos.col > 25 {
            result.push_str("...");
            result.push_str(&curr_line[pos.col-25 .. pos.col + len].to_string());
        } else {
            result.push_str(&curr_line[ .. pos.col + len].to_string());
        }
        res_pos = result.len() - len;

        if curr_line.len() - (pos.col + len) > 25 {
            result.push_str(&curr_line[pos.col + len .. pos.col + len + 24].to_string()); 
            result.push_str("...");
        } else {
            result.push_str(&curr_line[pos.col + len ..].to_string());    
        }

        Ok( (result, res_pos) )
    }
}
