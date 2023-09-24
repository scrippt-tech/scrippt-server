use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref PARSER: &'static str = r#"You are an extremely accurate {{record}} parser. When you get a {{record}}, you need to clean the text and extract the following information in the following JSON format:
    {{format}}
    Reply with only the answer in JSON format and include no other commentary
    
    Here is a {{record}} for you to parse:
    {{record_content}}
    
    Extracted information:
    ```json"#;
    pub(crate) static ref RESPONSE: &'static str = r#"You are a candidate who is applying for a job at a company. Following you will receive some highlights about your background. You will then then receive a prompt that you will need to answer. Your answer should highlight your strengths and experience.

    Experience:
    {{experience}}
    Education:
    {{education}}
    Skills:
    {{skills}}
    This is additional information you may use to help answer the question:
    {{additional}}
    
    Here is the prompt you will need to answer:
    {{prompt}}"#;
}
