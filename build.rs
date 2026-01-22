use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // reload font.txt and build.rs if they changed
    println!("cargo:rerun-if-changed=font.txt");
    println!("cargo:rerun-if-changed=build.rs");

    // load font.txt
    let input =
        fs::read_to_string("font.txt").expect("Failed to read font.txt");
    let lines: Vec<&str> = input
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // make font_table
    let mut font_table = [[['.'; 8]; 16]; 256];
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        let char_code_str =
            line.trim_start_matches("0x").trim_start_matches("0X");
        if let Ok(char_code) = usize::from_str_radix(char_code_str, 16) {
            if char_code >= 256 {
                break;
            }

            i += 1;

            for (y, bitmap_line) in lines.iter().skip(i).take(16).enumerate() {
                let row_chars: Vec<char> = bitmap_line.chars().collect();
                let copy_line = row_chars.len().min(8);
                font_table[char_code][y][..copy_line]
                    .copy_from_slice(&row_chars[..copy_line]);
            }

            i += 16;
        }
    }

    generate_rust_code(&font_table);
}

fn generate_rust_code(font_memory: &[[[char; 8]; 16]; 256]) {
    let mut output = String::new();
    output.push_str("static FONT_TABLE: [[[char; 8]; 16]; 256] = [\n");

    for char_data in font_memory.iter() {
        output.push_str("    [\n");
        for row in char_data.iter() {
            output.push_str("        [");
            for (x, &c) in row.iter().enumerate() {
                output.push_str(&format!("'{}'", c));
                if x < 7 {
                    output.push_str(", ");
                }
            }
            output.push_str("],\n");
        }
        output.push_str("    ],\n");
    }
    output.push_str("];\n");

    let dest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("font_table.rs");
    fs::write(dest_path, output).expect("Failed to write font_table.rs");
}
