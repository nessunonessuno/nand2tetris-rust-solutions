use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;

enum InstructionType {
    A(u16),  // A instruction with a number
    C { dest: Option<String>, comp: String, jump: Option<String> },
}

fn main() {
    let mut instructions = vec![
        String::from("// Initialize"),
        String::from("@2"),
        String::from("D=A"),
        String::from("@3"),
        String::from("D=D+A"),
        String::from("@0"),
        String::from("M=D"),
    ];
    let args: Vec<String> = env::args().collect();
    let filepath;
    if args.len() > 1 {
        filepath = args[1].clone();
        instructions = read_file(&filepath);
    }

    let (_symbol_table, parsed_instructions) = check_for_symbol_and_parse(instructions);
    translate_to_binary(parsed_instructions);
}

fn check_for_symbol_and_parse(instructions: Vec<String>) -> (HashMap<String, u16>, Vec<InstructionType>) {
    let mut symbol_table: HashMap<String, u16> = HashMap::new();
    let mut parsed = Vec::new();
    let mut rom_address = 0u16; 
    let mut ram_address = 16u16; 

// I spent some time figuring out why my binary output was slightly different from the original, despite the correct execution. Eventually, I discovered that they had hardcoded certain memory spaces.
// So, I created a table for their values. Before this, I didn't perform this step and simply allocated memory space for the values in sequential order.
// I guess im a turd or they are idk, but in general this part is not obligatory
    let predefined_symbols = [
        ("SP", 0), ("LCL", 1), ("ARG", 2), ("THIS", 3), ("THAT", 4),
        ("R0", 0), ("R1", 1), ("R2", 2), ("R3", 3), ("R4", 4), ("R5", 5),
        ("R6", 6), ("R7", 7), ("R8", 8), ("R9", 9), ("R10", 10), ("R11", 11),
        ("R12", 12), ("R13", 13), ("R14", 14), ("R15", 15), ("SCREEN", 16384),
        ("KBD", 24576)
    ];
    
    for &(symbol, address) in &predefined_symbols {
        symbol_table.insert(symbol.to_string(), address);
    }

    // First pass: handle labels
    for line in instructions.iter() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("(") && trimmed_line.ends_with(")") {
            let symbol = trimmed_line[1..trimmed_line.len() - 1].to_string();
            symbol_table.insert(symbol, rom_address);
        } else if !trimmed_line.starts_with("//") && !trimmed_line.is_empty() {
            rom_address += 1;
        }
    }

    // Second pass: handle other instructions
    for line in instructions.iter() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("@") {
            let symbol = trimmed_line[1..].to_string();
            if let Ok(value) = symbol.parse::<u16>() {
                parsed.push(InstructionType::A(value));
            } else {
                if !symbol_table.contains_key(&symbol) {
                    symbol_table.insert(symbol.clone(), ram_address);
                    parsed.push(InstructionType::A(ram_address));
                    ram_address += 1;
                } else {
                    parsed.push(InstructionType::A(*symbol_table.get(&symbol).unwrap()));
                }
            }
        } else if !trimmed_line.starts_with("(") && !trimmed_line.starts_with("//") && !trimmed_line.is_empty() {
            let (dest, comp, jump) = parse_c_instruction(trimmed_line);
            parsed.push(InstructionType::C { dest, comp, jump });
        }
    }

    (symbol_table, parsed)
}

fn translate_to_binary(instructions: Vec<InstructionType>) {
    let mut binary_result_file = fs::File::create("result.hack").unwrap();

    let comp_table = HashMap::from([
        ("0", "0101010"), ("1", "0111111"), ("-1", "0111010"), ("D", "0001100"),
        ("A", "0110000"), ("!D", "0001101"), ("!A", "0110001"), ("-D", "0001111"),
        ("-A", "0110011"), ("D+1", "0011111"), ("A+1", "0110111"), ("D-1", "0001110"),
        ("A-1", "0110010"), ("D+A", "0000010"), ("D-A", "0010011"), ("A-D", "0000111"),
        ("D&A", "0000000"), ("D|A", "0010101"), ("M", "1110000"), ("!M", "1110001"),
        ("-M", "1110011"), ("M+1", "1110111"), ("M-1", "1110010"), ("D+M", "1000010"),
        ("D-M", "1010011"), ("M-D", "1000111"), ("D&M", "1000000"), ("D|M", "1010101"),
    ]);

    let dest_table = HashMap::from([
        ("null", "000"), ("M", "001"), ("D", "010"), ("MD", "011"),
        ("A", "100"), ("AM", "101"), ("AD", "110"), ("AMD", "111"),
    ]);

    let jump_table = HashMap::from([
        ("null", "000"), ("JGT", "001"), ("JEQ", "010"), ("JGE", "011"),
        ("JLT", "100"), ("JNE", "101"), ("JLE", "110"), ("JMP", "111"),
    ]);

    for instruction in instructions {
        let binary_instruction = match instruction {
            InstructionType::A(value) => format!("{:016b}", value),
            InstructionType::C { dest, comp, jump } => {
                let comp_bits = comp_table.get(comp.as_str()).unwrap();
                let dest_bits = dest_table.get(dest.as_deref().unwrap_or("null")).unwrap();
                let jump_bits = jump_table.get(jump.as_deref().unwrap_or("null")).unwrap();
                format!("111{}{}{}", comp_bits, dest_bits, jump_bits)
            }
        };

        writeln!(binary_result_file, "{}", binary_instruction).unwrap();
    }
}

fn parse_c_instruction(instruction: &str) -> (Option<String>, String, Option<String>) {
    let mut dest = None;
    let mut comp = instruction.to_string();
    let mut jump = None;

    if let Some(jump_idx) = instruction.find(';') {
        jump = Some(instruction[jump_idx + 1..].to_string());
        comp = instruction[..jump_idx].to_string();
    }

    if let Some(dest_idx) = comp.find('=') {
        dest = Some(comp[..dest_idx].to_string());
        comp = comp[dest_idx + 1..].to_string();
    }

    (dest, comp, jump)
}

fn read_file(filepath: &str) -> Vec<String> {
    let contents = fs::read_to_string(filepath)
        .expect("Should have been able to read the file");
    let split_content: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
    split_content
}
