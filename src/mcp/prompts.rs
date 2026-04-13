pub struct Prompt {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: Vec<PromptArgument>,
}

pub struct PromptArgument {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

pub fn get_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "analyze_error",
            description: "Analyze a compiler error and suggest fixes",
            arguments: vec![
                PromptArgument {
                    name: "file_path",
                    description: "Path to the file with the error",
                    required: true,
                },
                PromptArgument {
                    name: "error_message",
                    description: "The error message from the compiler",
                    required: true,
                },
            ],
        },
        Prompt {
            name: "explain_code",
            description: "Explain what a piece of Rust code does",
            arguments: vec![
                PromptArgument {
                    name: "file_path",
                    description: "Path to the file",
                    required: true,
                },
                PromptArgument {
                    name: "line_start",
                    description: "Starting line number",
                    required: true,
                },
                PromptArgument {
                    name: "line_end",
                    description: "Ending line number",
                    required: false,
                },
            ],
        },
        Prompt {
            name: "find_similar",
            description: "Find similar code patterns in the codebase",
            arguments: vec![PromptArgument {
                name: "symbol_name",
                description: "Name of the symbol to find similar patterns for",
                required: true,
            }],
        },
    ]
}
