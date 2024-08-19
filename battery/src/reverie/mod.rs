use std::{fs::File, io::{BufRead, BufReader}};

pub mod memory;

pub fn generate_prompt(curr_input: Vec<String>, prompt_lib_file: &str) -> String {
    // Read the prompt file
    let file = File::open(prompt_lib_file).expect("Failed to open prompt file");
    let reader = BufReader::new(file);
    let prompt: String = reader.lines()
                            .map(|line| line.unwrap_or_default())
                            .collect::<Vec<_>>()
                            .join("\n");

    // Replace the placeholders with the actual inputs
    let mut result = prompt;
    for (count, input) in curr_input.iter().enumerate() {
        result = result.replace(&format!("!<INPUT {}>!", count), input);
    }

    // Remove the comment block marker if present
    if result.contains("<commentblockmarker>###</commentblockmarker>") {
        result = result.split("<commentblockmarker>###</commentblockmarker>")
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
    }

    // Return the final prompt
    result.trim().to_string()
}
