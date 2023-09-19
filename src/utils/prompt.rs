use std::fs::File;
use std::io::Read;

pub fn load_prompt(filename: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(format! {"./src/prompts/{}.prompt", filename})?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}
