use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use super::{create_prompt_input, UserScratch};

pub fn generate_prompt<T: AsRef<str> + ToString, P: AsRef<Path>>(
    curr_input: Vec<T>,
    prompt_lib_file: P,
) -> io::Result<String> {
    // Read the prompt file content
    let mut file = File::open(prompt_lib_file)?;
    let mut prompt = String::new();
    file.read_to_string(&mut prompt)?;

    // Replace placeholder with actual inputs
    for (count, input) in curr_input.iter().enumerate() {
        let placeholder = format!("!<INPUT {}>!", count);
        prompt = prompt.replace(&placeholder, input.as_ref());
    }

    // Split at comment block marker and return the latter part if exists
    let prompt_parts: Vec<&str> = prompt.split("<commentblockmarker>###</commentblockmarker>").collect();
    let final_prompt = if prompt_parts.len() > 1 { prompt_parts[1] } else { prompt_parts[0] };

    Ok(final_prompt.trim().to_string())
}

/*
Main cognitive function of the chain. It takes the retrieved memory and 
perception, as well as the maze and the first day state to conduct both 
the long term and short term planning for the persona. 

INPUT: 
  maze: Current <Maze> instance of the world. 
  personas: A dictionary that contains all persona names as keys, and the 
            Persona instance as values. 
  new_day: This can take one of the three values. 
    1) <Boolean> False -- It is not a "new day" cycle (if it is, we would
       need to call the long term planning sequence for the persona). 
    2) <String> "First day" -- It is literally the start of a simulation,
       so not only is it a new day, but also it is the first day. 
    2) <String> "New day" -- It is a new day. 
  retrieved: dictionary of dictionary. The first layer specifies an event,
             while the latter layer specifies the "curr_event", "events", 
             and "thoughts" that are relevant.
OUTPUT 
  The target action address of the persona (persona.scratch.act_address).

  */
pub fn plan<T: AsRef<str> + ToString, P: AsRef<Path>>(
    maze: Vec<String>,
    personas: HashMap<String, UserScratch>,
    new_day: T,
    retrieved: Option<Vec<&str>>,
    prompt_lib_file: P,
) -> String {
    // Conduct long term planning if it is a new day
    if new_day.to_string() == "First day" || new_day.to_string() == "New day" {
        for (name, persona) in personas.iter() {
            // persona.scratch.act_address = run_gpt_prompt_daily_plan(&persona, 6).unwrap();
        }
    }

    // Conduct short term planning
    for (name, persona) in personas.iter() {
        let curr_input = create_prompt_input(persona, retrieved.clone());
        let prompt = generate_prompt(curr_input, &prompt_lib_file).unwrap();
        // persona.scratch.act_address = run_gpt_prompt(&persona, prompt).unwrap();
    }

    // Return the target action address of the persona
    personas["persona_name"].act_address.clone()
}
