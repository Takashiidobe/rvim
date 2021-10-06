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
        if file_name.ends_with(".c") {
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
                        "struct", "switch", "typedef", "union", "void", "volatile", "while",
                        "#include", "#ifndef", "#if", "#endif", "#define", "#undef"
                    ],
                    secondary_keywords: str_vec![
                        "char",
                        "int",
                        "long",
                        "unsigned",
                        "float",
                        "double",
                        "size_t",
                        "signed",
                        "short",
                        "wchar_t",
                        "__int128_t",
                        "bool"
                    ],
                },
            };
        } else if file_name.ends_with(".cc")
            || file_name.ends_with(".cpp")
            || file_name.ends_with(".C")
            || file_name.ends_with(".h")
            || file_name.ends_with(".hh")
            || file_name.ends_with(".hpp")
        {
            return Self {
                name: String::from("C++"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: str_vec![
                        "alignas",
                        "alignof",
                        "and",
                        "and_eq",
                        "asm",
                        "atomic_cancel",
                        "atomic_commit",
                        "atomic_noexcept",
                        "auto",
                        "bitand",
                        "bitor",
                        "bool",
                        "break",
                        "case",
                        "catch",
                        "char",
                        "char8_t",
                        "char16_t",
                        "char32_t",
                        "class",
                        "compl",
                        "concept",
                        "const",
                        "consteval",
                        "constexpr",
                        "constinit",
                        "const_cast",
                        "continue",
                        "co_await",
                        "co_return",
                        "co_yield",
                        "decltype",
                        "default",
                        "delete",
                        "do",
                        "double",
                        "dynamic_cast",
                        "else",
                        "enum",
                        "explicit",
                        "export",
                        "extern",
                        "false",
                        "float",
                        "for",
                        "friend",
                        "goto",
                        "if",
                        "inline",
                        "int",
                        "long",
                        "mutable",
                        "namespace",
                        "new",
                        "noexcept",
                        "not",
                        "not_eq",
                        "nullptr",
                        "operator",
                        "or",
                        "or_eq",
                        "private",
                        "protected",
                        "public",
                        "reflexpr",
                        "register",
                        "reinterpret_cast",
                        "requires",
                        "return",
                        "short",
                        "signed",
                        "sizeof",
                        "static",
                        "static_assert",
                        "static_cast",
                        "struct",
                        "switch",
                        "synchronized",
                        "template",
                        "this",
                        "thread_local",
                        "throw",
                        "true",
                        "try",
                        "typedef",
                        "typeid",
                        "typename",
                        "union",
                        "unsigned",
                        "using",
                        "virtual",
                        "void",
                        "volatile",
                        "wchar_t",
                        "while",
                        "xor",
                        "xor_eq"
                    ],
                    secondary_keywords: str_vec![
                        "char",
                        "int",
                        "long",
                        "unsigned",
                        "float",
                        "double",
                        "size_t",
                        "signed",
                        "short",
                        "wchar_t",
                        "__int128_t",
                        "bool"
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
                        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mut",
                        "pub", "ref", "return", "self", "Self", "static", "struct", "super",
                        "trait", "true", "type", "unsafe", "use", "where", "while", "dyn", "box",
                        "do", "final", "macro", "typeof", "unsized", "yield", "async", "await",
                        "try"
                    ],
                    secondary_keywords: str_vec![
                        "bool", "char", "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32",
                        "u64", "usize", "f32", "f64"
                    ],
                },
            };
        } else if file_name.ends_with(".js") {
            return Self {
                name: String::from("Javascript"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: str_vec![
                        "async",
                        "await",
                        "break",
                        "case",
                        "catch",
                        "class",
                        "const",
                        "continue",
                        "debugger",
                        "default",
                        "delete",
                        "do",
                        "else",
                        "export",
                        "extends",
                        "finally",
                        "for",
                        "function",
                        "if",
                        "import",
                        "in",
                        "instanceof",
                        "let",
                        "new",
                        "return",
                        "super",
                        "switch",
                        "this",
                        "throw",
                        "try",
                        "typeof",
                        "var",
                        "void",
                        "while",
                        "with",
                        "yield"
                    ],
                    secondary_keywords: str_vec!["get", "set"],
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
