use sendgrid::error::SendgridError;
use sendgrid::v3::*;
use std::collections::HashMap;

/// Send an email verification email to the user.
pub async fn send_email_verification(email: &str, name: &str, code: &str) -> Result<(), SendgridError> {
    let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = Sender::new(api_key);

    let mut template_data = HashMap::with_capacity(2);
    template_data.insert("name".to_string(), name.to_string());
    template_data.insert("code".to_string(), code.to_string());

    let personalization = Personalization::new(Email::new(email.to_string())).add_dynamic_template_data(template_data);

    let sender = Email::new("noreply@scrippt.tech".to_string()).set_name("Scrippt".to_string());
    let message = Message::new(sender)
        .set_subject("Scrippt: Verify your email!")
        .add_personalization(personalization)
        .set_template_id("d-2bef8acb2a844b15b5de389d8b8eea09");

    let resp = client.send(&message).await?;

    log::debug!("[SENDGRID] Verification code response email: {:?}", resp);

    Ok(())
}

/// Send a welcome email to the user.
/// TODO: Add template ID
pub async fn send_account_created(email: &str, name: &str) -> Result<(), SendgridError> {
    let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = Sender::new(api_key);

    let mut template_data = HashMap::with_capacity(1);
    template_data.insert("name".to_string(), name.to_string());

    let personalization = Personalization::new(Email::new(email.to_string())).add_dynamic_template_data(template_data);

    let sender = Email::new("noreply@scrippt.tech".to_string()).set_name("Scrippt".to_string());
    let message = Message::new(sender)
        .set_subject("Scrippt: Your account has been created!")
        .set_reply_to(Email::new("info@scrippt.tech".to_string()))
        .add_personalization(personalization)
        .set_template_id("d-44b622c643b34c29b2be87701fde4c6c");

    let resp = client.send(&message).await?;

    log::debug!("[SENDGRID] Account created response email: {:?}", resp);

    Ok(())
}

/// Send contact email to info@scrippt.tech
pub async fn send_contact_email(name: &str, email: &str, message: &str) -> Result<(), SendgridError> {
    let api_key = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = Sender::new(api_key);

    let mut template_data = HashMap::with_capacity(3);
    template_data.insert("name".to_string(), name.to_string());
    template_data.insert("email".to_string(), email.to_string());
    template_data.insert("message".to_string(), message.to_string());

    let personalization = Personalization::new(Email::new("info@scrippt.tech".to_string())).add_dynamic_template_data(template_data);
    let sender = Email::new("noreply@scrippt.tech".to_string()).set_name("[CONTACT FORM] Scrippt".to_string());
    let message = Message::new(sender)
        .set_subject("[CONTACT FORM] From [{{name}} - {{email}}]")
        .set_reply_to(Email::new(email.to_string()))
        .add_personalization(personalization);

    let resp = client.send(&message).await?;

    log::debug!("[SENDGRID] Contact form response email: {:?}", resp);

    Ok(())
}
