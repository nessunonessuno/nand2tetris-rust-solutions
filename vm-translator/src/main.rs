use std::fs;
use std::env;

use std::sync::atomic::{AtomicUsize, Ordering};

static LABEL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("Usage: {} <file.vm>", args[0]);
        std::process::exit(1);
    }
    let filepath = &args[1];

    match fs::read_to_string(filepath) {
        Ok(file_content) => {
            let parsed_content = parser(&file_content);
            let asm_result = convert_to_asm(&parsed_content);
            let output_filepath = filepath.replace(".vm", ".asm");
            if write_file_asm(&asm_result, &output_filepath).is_err() {
                eprintln!("Failed to write to file: {}", output_filepath);
                std::process::exit(1);
            }
            println!("Translation completed successfully: {}", output_filepath);
        },
        Err(e) => {
            eprintln!("Failed to read the file '{}': {}", filepath, e);
            std::process::exit(1);
        }
    }
}

fn parser(file_content: &str) -> Vec<CommandType> {
    file_content.lines()
        .filter_map(parse_line)
        .collect()
}


fn convert_to_asm(parsed_content: &[CommandType]) -> String {
    let mut asm_result = String::new();
    for command in parsed_content {
        match command {
            CommandType::Push(segment, index) => {
                let asm_code = match segment.as_str() {
                    "constant" => format!(
                        "@{index}\n\
                        D=A\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n"
                    ),
                    "local" => format!(
                        "@{index}\n\
                        D=A\n\
                        @LCL\n\
                        A=M+D\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n"
                    ),
                    "argument" => format!(
                        "@{index}\n\
                        D=A\n\
                        @ARG\n\
                        A=M+D\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n"
                    ),
                    "this" => format!(
                        "@{index}\n\
                        D=A\n\
                        @THIS\n\
                        A=M+D\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n"
                    ),
                    "that" => format!(
                        "@{index}\n\
                        D=A\n\
                        @THAT\n\
                        A=M+D\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n"
                    ),
                    "temp" => format!(
                        "@{}\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n",
                        5 + index
                    ),
                    "pointer" => format!(
                        "@{}\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n",
                        if *index == 0 { "THIS" } else { "THAT" }
                    ),
                    "static" => format!(
                        "@{}.{}\n\
                        D=M\n\
                        @SP\n\
                        A=M\n\
                        M=D\n\
                        @SP\n\
                        M=M+1\n",
                        "Foo", index
                    ),
                    _ => panic!("Unsupported push segment: {}", segment),
                };
                asm_result.push_str(&asm_code);
            },
            CommandType::Pop(segment, index) => {
                let asm_code = match segment.as_str() {
                    "local" => format!(
                        "@{index}\n\
                        D=A\n\
                        @LCL\n\
                        D=M+D\n\
                        @R13\n\
                        M=D\n\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @R13\n\
                        A=M\n\
                        M=D\n"
                    ),
                    "argument" => format!(
                        "@{index}\n\
                        D=A\n\
                        @ARG\n\
                        D=M+D\n\
                        @R13\n\
                        M=D\n\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @R13\n\
                        A=M\n\
                        M=D\n"
                    ),
                    "this" => format!(
                        "@{index}\n\
                        D=A\n\
                        @THIS\n\
                        D=M+D\n\
                        @R13\n\
                        M=D\n\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @R13\n\
                        A=M\n\
                        M=D\n"
                    ),
                    "that" => format!(
                        "@{index}\n\
                        D=A\n\
                        @THAT\n\
                        D=M+D\n\
                        @R13\n\
                        M=D\n\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @R13\n\
                        A=M\n\
                        M=D\n"
                    ),
                    "temp" => format!(
                        "@SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @{}\n\
                        M=D\n",
                        5 + index
                    ),
                    "pointer" => format!(
                        "@SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @{}\n\
                        M=D\n",
                        if *index == 0 { "THIS" } else { "THAT" }
                    ),
                    "static" => format!(
                        "@SP\n\
                        AM=M-1\n\
                        D=M\n\
                        @{}.{}\n\
                        M=D\n",
                        "Foo", index
                    ),
                    _ => panic!("Unsupported pop segment: {}", segment),
                };
                asm_result.push_str(&asm_code);
            },
            CommandType::Arithmetic(operation) => {
                asm_result.push_str(&format_arithmetic(operation));
            },
            CommandType::Label(label) => {
                asm_result.push_str(&format!("({})\n", label));
            },
            CommandType::Goto(label) => {
                asm_result.push_str(&format!(
                    "@{label}\n\
                    0;JMP\n"
                ));
            },
            CommandType::If(label) => {
                asm_result.push_str(&format!(
                    "@SP\n\
                    AM=M-1\n\
                    D=M\n\
                    @{label}\n\
                    D;JNE\n"
                ));
            },
            CommandType::Function(name, num_locals) => {
                asm_result.push_str(&format!("({name})\n"));
                for _ in 0..*num_locals {
                    asm_result.push_str(
                        "@SP\n\
                        A=M\n\
                        M=0\n\
                        @SP\n\
                        M=M+1\n"
                    );
                }
            },
            CommandType::Return => {
                asm_result.push_str(
                    "@LCL\n\
                    D=M\n\
                    @R14\n\
                    M=D\n\
                    @5\n\
                    A=D-A\n\
                    D=M\n\
                    @R15\n\
                    M=D\n\
                    @SP\n\
                    A=M-1\n\
                    D=M\n\
                    @ARG\n\
                    A=M\n\
                    M=D\n\
                    D=A+1\n\
                    @SP\n\
                    M=D\n\
                    @R14\n\
                    AM=M-1\n\
                    D=M\n\
                    @THAT\n\
                    M=D\n\
                    @R14\n\
                    AM=M-1\n\
                    D=M\n\
                    @THIS\n\
                    M=D\n\
                    @R14\n\
                    AM=M-1\n\
                    D=M\n\
                    @ARG\n\
                    M=D\n\
                    @R14\n\
                    AM=M-1\n\
                    D=M\n\
                    @LCL\n\
                    M=D\n\
                    @R15\n\
                    A=M\n\
                    0;JMP\n"
                );
            },
            CommandType::Call(name, num_args) => {
                let return_label = unique_label("RETURN_LABEL"); // Generate a unique return label
                asm_result.push_str(&format!(
                    "@{return_label}\n\
                    D=A\n\
                    @SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n\
                    @LCL\n\
                    D=M\n\
                    @SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n\
                    @ARG\n\
                    D=M\n\
                    @SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n\
                    @THIS\n\
                    D=M\n\
                    @SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n\
                    @THAT\n\
                    D=M\n\
                    @SP\n\
                    A=M\n\
                    M=D\n\
                    @SP\n\
                    M=M+1\n\
                    @SP\n\
                    D=M\n\
                    @5\n\
                    D=D-A\n\
                    @{num_args}\n\
                    D=D-A\n\
                    @ARG\n\
                    M=D\n\
                    @SP\n\
                    D=M\n\
                    @LCL\n\
                    M=D\n\
                    @{name}\n\
                    0;JMP\n\
                    ({return_label})\n"
                ));
            },
        }
    }
    asm_result
}

fn parse_line(line: &str) -> Option<CommandType> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("//") {
        None
    } else {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        match parts[0] {
            "push" if parts.len() == 3 => parts[2].parse::<i16>().ok().map(|index| CommandType::Push(parts[1].to_string(), index)),
            "pop" if parts.len() == 3 => parts[2].parse::<i16>().ok().map(|index| CommandType::Pop(parts[1].to_string(), index)),
            "label" if parts.len() == 2 => Some(CommandType::Label(parts[1].to_string())),
            "goto" if parts.len() == 2 => Some(CommandType::Goto(parts[1].to_string())),
            "if-goto" if parts.len() == 2 => Some(CommandType::If(parts[1].to_string())),
            "function" if parts.len() == 3 => parts[2].parse::<usize>().ok().map(|num_args| CommandType::Function(parts[1].to_string(), num_args)),
            "call" if parts.len() == 3 => parts[2].parse::<usize>().ok().map(|num_args| CommandType::Call(parts[1].to_string(), num_args)),
            "return" if parts.len() == 1 => Some(CommandType::Return),
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" if parts.len() == 1 => Some(CommandType::Arithmetic(parts[0].to_string())),
            _ => None,
        }
    }
}


fn format_arithmetic(operation: &str) -> String {
    match operation {
        "add" => {
            String::from(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                M=D+M\n"
            )
        },
        "sub" => {
            String::from(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                M=M-D\n"
            )
        },
        "neg" => {
            String::from(
                "@SP\n\
                A=M-1\n\
                M=-M\n"
            )
        },
        "eq" => {
            let label = unique_label("EQ");
            format!(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                D=M-D\n\
                M=-1\n\
                @{}\n\
                D;JEQ\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                ({})

                ", label, label
            )
        },
        "gt" => {
            let label = unique_label("GT");
            format!(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                D=M-D\n\
                M=-1\n\
                @{}\n\
                D;JGT\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                ({})

                ", label, label
            )
        },
        "lt" => {
            let label = unique_label("LT");
            format!(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                D=M-D\n\
                M=-1\n\
                @{}\n\
                D;JLT\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                ({})

                ", label, label
            )
        },
        "and" => {
            String::from(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                M=D&M\n"
            )
        },
        "or" => {
            String::from(
                "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                M=D|M\n"
            )
        },
        "not" => {
            String::from(
                "@SP\n\
                A=M-1\n\
                M=!M\n"
            )
        },
        _ => panic!("Unsupported arithmetic operation: {}", operation),
    }
}

fn unique_label(base: &str) -> String {
    let count = LABEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}{}", base, count)
}

fn write_file_asm(file_content: &str, filename: &str) -> Result<(), std::io::Error> {
    fs::write(filename, file_content)
}

enum CommandType {
    Arithmetic(String),
    Push(String, i16),
    Pop(String, i16),
    Label(String),
    Goto(String),
    If(String),
    Function(String, usize),
    Return,
    Call(String, usize),
}
