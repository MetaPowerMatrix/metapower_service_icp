use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize,Clone)]
pub struct Persona {
    pub name: String,
    pub age: u32,
    pub innate: String,
    pub learned: String,
    pub currently: String,
    pub lifestyle: String,
    pub daily_plan_req: String,
}

impl Persona {
    pub fn new(name: String, age: u32, innate: String, learned: String, currently: String, lifestyle: String, daily_plan_req: String) -> Self {
        Persona {
            name,
            age,
            innate,
            learned,
            currently,
            lifestyle,
            daily_plan_req,
        }
    }

    pub fn get_str_iss(&self) -> String {
        let mut commonset = String::new();
        commonset += &format!("{} is ", self.name);
        commonset += &format!("{} years old\n", self.age);
        commonset += &format!("Innate traits: {}\n", self.innate);
        // commonset += &format!("Learned traits: {}\n", self.learned);
        // commonset += &format!("Currently: {}\n", self.currently);
        commonset += &format!("Lifestyle: {}\n", self.lifestyle);
        // commonset += &format!("Daily plan requirement: {}\n", self.daily_plan_req);
        commonset
    }
}
