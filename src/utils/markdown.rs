use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel, CodeBlockKind};
use std::io::{self, Write};

pub struct MarkdownStreamRenderer {
    last_printed_byte_index: usize,
    in_bold: bool,
    in_italic: bool,
    in_code_block: bool,
    current_code_block_lang: Option<String>,
    in_blockquote: bool,
    in_list_item: bool,
    list_item_index: Option<u64>,
    header_level: Option<HeadingLevel>,
    current_column: usize,
    term_width: usize,
}

impl MarkdownStreamRenderer {
    pub fn new() -> Self {
        let term_width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80)
            .saturating_sub(4);
        Self {
            last_printed_byte_index: 0,
            in_bold: false,
            in_italic: false,
            in_code_block: false,
            current_code_block_lang: None,
            in_blockquote: false,
            in_list_item: false,
            list_item_index: None,
            header_level: None,
            current_column: 0,
            term_width,
        }
    }

    pub fn render_chunk(&mut self, text: &str) {
        let parser = Parser::new(text);
        let mut stdout = io::stdout();

        for (event, range) in parser.into_offset_iter() {
            let is_new = range.start >= self.last_printed_byte_index;
            let is_partial = range.start < self.last_printed_byte_index && range.end > self.last_printed_byte_index;

            match &event {
                Event::Start(tag) => {
                    self.update_state_start(tag);
                    if is_new {
                        self.print_tag_start(tag, &mut stdout);
                    }
                }
                Event::End(tag) => {
                    if is_new {
                        self.print_tag_end(tag, &mut stdout);
                    }
                    self.update_state_end(tag);
                }
                Event::Text(content) => {
                    if is_new {
                        self.print_styled_text(content, &mut stdout);
                    } else if is_partial {
                        let printed_len_in_this_event = self.last_printed_byte_index - range.start;
                        if printed_len_in_this_event < content.len() {
                            let new_text = &content[printed_len_in_this_event..];
                            self.print_styled_text(new_text, &mut stdout);
                        }
                    }
                }
                Event::Code(content) => {
                    if is_new {
                        self.print_inline_code(content, &mut stdout);
                    } else if is_partial {
                        let printed_len_in_this_event = self.last_printed_byte_index - range.start;
                        if printed_len_in_this_event < content.len() {
                            let new_text = &content[printed_len_in_this_event..];
                            self.print_inline_code(new_text, &mut stdout);
                        }
                    }
                }
                Event::SoftBreak => {
                    if is_new || is_partial {
                        self.print_soft_break(&mut stdout);
                    }
                }
                Event::HardBreak => {
                    if is_new || is_partial {
                        self.print_hard_break(&mut stdout);
                    }
                }
                Event::Rule => {
                    if is_new {
                        self.print_rule(&mut stdout);
                    }
                }
                _ => {}
            }

            if range.end > self.last_printed_byte_index {
                self.last_printed_byte_index = range.end;
            }
        }
        let _ = stdout.flush();
    }

    fn update_state_start(&mut self, tag: &Tag) {
        match tag {
            Tag::Strong => self.in_bold = true,
            Tag::Emphasis => self.in_italic = true,
            Tag::BlockQuote(_) => self.in_blockquote = true,
            Tag::CodeBlock(kind) => {
                self.in_code_block = true;
                if let CodeBlockKind::Fenced(lang) = kind {
                    self.current_code_block_lang = Some(lang.to_string());
                } else {
                    self.current_code_block_lang = None;
                }
            }
            Tag::Heading { level, .. } => self.header_level = Some(*level),
            Tag::Item => self.in_list_item = true,
            Tag::List(start_num) => self.list_item_index = *start_num,
            _ => {}
        }
    }

    fn update_state_end(&mut self, tag: &TagEnd) {
        match tag {
            TagEnd::Strong => self.in_bold = false,
            TagEnd::Emphasis => self.in_italic = false,
            TagEnd::BlockQuote(_) => self.in_blockquote = false,
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.current_code_block_lang = None;
            }
            TagEnd::Heading(_) => self.header_level = None,
            TagEnd::Item => self.in_list_item = false,
            TagEnd::List(_) => self.list_item_index = None,
            _ => {}
        }
    }

    fn print_tag_start(&mut self, tag: &Tag, stdout: &mut io::Stdout) {
        match tag {
            Tag::BlockQuote(_) => {
                let _ = write!(stdout, "\r\n\x1b[38;2;98;114;164m│ \x1b[0m");
                self.current_column = 2;
            }
            Tag::CodeBlock(_) => {
                let _ = write!(stdout, "\r\n\x1b[38;2;98;114;164m┌──────────────────────────────────────────────────\x1b[0m\r\n");
                self.current_column = 0;
            }
            Tag::Heading { level, .. } => {
                let prefix = match level {
                    HeadingLevel::H1 => "\r\n\x1b[1;38;2;255;121;198m# ",
                    HeadingLevel::H2 => "\r\n\x1b[1;38;2;139;233;253m## ",
                    HeadingLevel::H3 => "\r\n\x1b[1;38;2;189;147;249m### ",
                    _ => "\r\n\x1b[1m#### ",
                };
                let _ = write!(stdout, "{}", prefix);
                self.current_column = 0;
            }
            Tag::Item => {
                if let Some(num) = self.list_item_index {
                    let prefix = format!(" {}. ", num);
                    let _ = write!(stdout, "\r\n\x1b[38;2;255;121;198m{}\x1b[0m", prefix);
                    self.current_column = prefix.chars().count();
                    self.list_item_index = Some(num + 1);
                } else {
                    let _ = write!(stdout, "\r\n\x1b[38;2;255;121;198m• \x1b[0m");
                    self.current_column = 2;
                }
            }
            _ => {}
        }
    }

    fn print_tag_end(&mut self, tag: &TagEnd, stdout: &mut io::Stdout) {
        match tag {
            TagEnd::BlockQuote(_) => {
                let _ = write!(stdout, "\r\n");
                self.current_column = 0;
            }
            TagEnd::CodeBlock => {
                let _ = write!(stdout, "\x1b[38;2;98;114;164m└──────────────────────────────────────────────────\x1b[0m\r\n");
                self.current_column = 0;
            }
            TagEnd::Heading(_) => {
                let _ = write!(stdout, "\x1b[0m\r\n");
                self.current_column = 0;
            }
            _ => {}
        }
    }

    fn print_styled_text(&mut self, text: &str, stdout: &mut io::Stdout) {
        if self.in_code_block {
            let mut highlighted_lines = Vec::new();
            for line in text.split('\n') {
                highlighted_lines.push(highlight_code_line(line));
            }
            let joined = highlighted_lines.join("\r\n");
            let _ = write!(stdout, "{}", joined);
            return;
        }

        let words = text.split_inclusive(|c: char| c.is_whitespace());
        for word in words {
            if word.contains('\n') {
                let parts: Vec<&str> = word.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        let _ = write!(stdout, "\r\n");
                        if self.in_blockquote {
                            let _ = write!(stdout, "\x1b[38;2;98;114;164m│ \x1b[0m");
                        }
                        self.current_column = if self.in_blockquote { 2 } else { 0 };
                    }
                    if !part.is_empty() {
                        self.print_word(part, stdout);
                    }
                }
            } else {
                self.print_word(word, stdout);
            }
        }
    }

    fn print_word(&mut self, word: &str, stdout: &mut io::Stdout) {
        let clean_word_len = word.chars().count();
        if self.current_column + clean_word_len > self.term_width {
            let _ = write!(stdout, "\r\n");
            if self.in_blockquote {
                let _ = write!(stdout, "\x1b[38;2;98;114;164m│ \x1b[0m");
            }
            self.current_column = if self.in_blockquote { 2 } else { 0 };
            
            let trimmed = word.trim_start();
            self.print_styled_span(trimmed, stdout);
            self.current_column += trimmed.chars().count();
        } else {
            self.print_styled_span(word, stdout);
            self.current_column += clean_word_len;
        }
    }

    fn print_styled_span(&mut self, text: &str, stdout: &mut io::Stdout) {
        let pink = "\x1b[1;38;2;255;121;198m";
        let cyan = "\x1b[3;38;2;139;233;253m";
        let reset = "\x1b[0m";

        let mut styled = String::new();
        if self.in_bold {
            styled.push_str(pink);
        }
        if self.in_italic {
            styled.push_str(cyan);
        }
        if let Some(level) = self.header_level {
            match level {
                HeadingLevel::H1 => styled.push_str("\x1b[1;38;2;255;121;198m"),
                HeadingLevel::H2 => styled.push_str("\x1b[1;38;2;139;233;253m"),
                HeadingLevel::H3 => styled.push_str("\x1b[1;38;2;189;147;249m"),
                _ => styled.push_str("\x1b[1m"),
            }
        }
        styled.push_str(text);
        if self.in_bold || self.in_italic || self.header_level.is_some() {
            styled.push_str(reset);
        }
        let _ = write!(stdout, "{}", styled);
    }

    fn print_inline_code(&mut self, text: &str, stdout: &mut io::Stdout) {
        let purple_bg = "\x1b[48;2;40;42;54;38;2;189;147;249m";
        let reset = "\x1b[0m";
        let _ = write!(stdout, "{}`{}`{}", purple_bg, text, reset);
        self.current_column += text.chars().count() + 2;
    }

    fn print_soft_break(&mut self, stdout: &mut io::Stdout) {
        let _ = write!(stdout, " ");
        self.current_column += 1;
    }

    fn print_hard_break(&mut self, stdout: &mut io::Stdout) {
        let _ = write!(stdout, "\r\n");
        if self.in_blockquote {
            let _ = write!(stdout, "\x1b[38;2;98;114;164m│ \x1b[0m");
        }
        self.current_column = if self.in_blockquote { 2 } else { 0 };
    }

    fn print_rule(&mut self, stdout: &mut io::Stdout) {
        let _ = write!(stdout, "\r\n\x1b[38;2;98;114;164m──────────────────────────────────────────────────\x1b[0m\r\n");
        self.current_column = 0;
    }
}

fn highlight_code_line(line: &str) -> String {
    let mut highlighted = String::new();
    let mut chars = line.chars().peekable();
    
    let pink = "\x1b[38;2;255;121;198m";
    let yellow = "\x1b[38;2;241;250;140m";
    let orange = "\x1b[38;2;255;184;108m";
    let cyan = "\x1b[38;2;139;233;253m";
    let comment_color = "\x1b[38;2;98;114;164m";
    let reset = "\x1b[0m";

    while let Some(&c) = chars.peek() {
        if c == '/' {
            chars.next();
            if chars.peek() == Some(&'/') {
                highlighted.push_str(comment_color);
                highlighted.push('/');
                highlighted.push('/');
                while let Some(c) = chars.next() {
                    highlighted.push(c);
                }
                highlighted.push_str(reset);
                break;
            } else {
                highlighted.push('/');
            }
        } else if c == '#' {
            highlighted.push_str(comment_color);
            while let Some(c) = chars.next() {
                highlighted.push(c);
            }
            highlighted.push_str(reset);
            break;
        } else if c == '"' || c == '\'' {
            let quote = chars.next().unwrap();
            highlighted.push_str(yellow);
            highlighted.push(quote);
            let mut escaped = false;
            while let Some(&nc) = chars.peek() {
                if escaped {
                    highlighted.push(chars.next().unwrap());
                    escaped = false;
                } else if nc == '\\' {
                    highlighted.push(chars.next().unwrap());
                    escaped = true;
                } else if nc == quote {
                    highlighted.push(chars.next().unwrap());
                    break;
                } else {
                    highlighted.push(chars.next().unwrap());
                }
            }
            highlighted.push_str(reset);
        } else if c.is_ascii_digit() {
            highlighted.push_str(orange);
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' {
                    highlighted.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            highlighted.push_str(reset);
        } else if c.is_alphabetic() || c == '_' {
            let mut word = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            match word.as_str() {
                "fn" | "let" | "mut" | "pub" | "struct" | "impl" | "enum" | "use" | "mod" | "crate" |
                "def" | "class" | "import" | "from" | "as" | "return" | "if" | "else" | "elif" |
                "for" | "while" | "in" | "match" | "break" | "continue" | "const" | "var" | "function" |
                "interface" | "type" | "export" | "default" | "self" | "Self" => {
                    highlighted.push_str(pink);
                    highlighted.push_str(&word);
                    highlighted.push_str(reset);
                }
                "true" | "false" | "None" | "Some" | "Ok" | "Err" | "nil" | "null" => {
                    highlighted.push_str(orange);
                    highlighted.push_str(&word);
                    highlighted.push_str(reset);
                }
                "println" | "print" | "log" | "console" | "std" | "io" | "anyhow" | "Result" | "Option" | "Vec" | "String" => {
                    highlighted.push_str(cyan);
                    highlighted.push_str(&word);
                    highlighted.push_str(reset);
                }
                _ => {
                    highlighted.push_str(&word);
                }
            }
        } else {
            highlighted.push(chars.next().unwrap());
        }
    }
    highlighted
}
