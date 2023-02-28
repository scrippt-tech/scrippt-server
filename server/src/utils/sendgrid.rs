use sendgrid::error::SendgridError;
use sendgrid::v3::*;
use std::collections::HashMap;

pub async fn send_email_verification(
    email: &str,
    name: &str,
    code: &str,
) -> Result<(), SendgridError> {
    let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = Sender::new(api_key);

    let mut template_data = HashMap::with_capacity(2);
    template_data.insert("name".to_string(), name.to_string());
    template_data.insert("code".to_string(), code.to_string());

    let personalization = Personalization::new(Email::new(email.to_string()))
        .add_dynamic_template_data(template_data);

    let message = Message::new(Email::new("noreply@scrippt.tech".to_string()))
        .set_subject("Scrippt: Verify your email!")
        .add_personalization(personalization)
        .set_template_id("d-2bef8acb2a844b15b5de389d8b8eea09");

    let resp = client.send(&message).await?;
    println!("{:?}", resp);

    Ok(())
}
