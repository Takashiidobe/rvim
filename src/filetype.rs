pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

#[derive(Default)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiline_comments: bool,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

macro_rules! str_vec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn highlighting_options(&self) -> &HighlightingOptions {
        &self.hl_opts
    }
    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".c") || file_name.ends_with(".h") {
            return Self {
                name: String::from("C"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: str_vec![
                        "auto", "break", "case", "const", "continue", "default", "do", "enum",
                        "extern", "for", "goto", "if", "register", "return", "sizeof", "static",
                        "struct", "switch", "typedef", "union", "void", "volatile", "while"
                    ],
                    secondary_keywords: str_vec![
                        "char", "int", "long", "unsigned", "float", "double", "size_t", "signed",
                        "short", "#include", "#ifndef", "#if", "#endif", "#define", "#undef"
                    ],
                },
            };
        } else if file_name.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: str_vec![
                        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                        "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                        "super", "trait", "true", "type", "unsafe", "use", "where", "while", "dyn",
                        "abstract", "become", "box", "do", "final", "macro", "override", "priv",
                        "typeof", "unsized", "virtual", "yield", "async", "await", "try"
                    ],
                    secondary_keywords: str_vec![
                        "bool", "char", "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32",
                        "u64", "usize", "f32", "f64"
                    ],
                },
            };
        }
        Self::default()
    }
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }
    pub fn strings(&self) -> bool {
        self.strings
    }
    pub fn characters(&self) -> bool {
        self.characters
    }
    pub fn comments(&self) -> bool {
        self.comments
    }
    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.primary_keywords
    }
    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.secondary_keywords
    }
    pub fn multiline_comments(&self) -> bool {
        self.multiline_comments
    }
}
