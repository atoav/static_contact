//! Mails are sent via SMTP connection, with a minimal layout. At the moment no alternative Ports or encryption modes are supported, but this will follow in the future (probably).

use crate::FormData;
use crate::config::Config;
use crate::config::EndpointConfig;
use lettre::smtp::response::Response;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{Transport, SmtpClient};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre_email::Email;

/// Send email via SMTP
pub fn send_mail(data: &FormData, config: &Config, endpoint_config: &EndpointConfig) -> Result<Response, lettre::smtp::error::Error> {
    // Add this to the text if the user entered a phone number
    let phone_contact = match data.phone.as_str() {
        "" => "".to_string(),
        _ => format!(" or call {}", data.phone)
    };

    // Construct Email
    let email = Email::builder()
        // Addresses can be specified by the tuple (email, alias)
        .to((endpoint_config.target.email.clone(), endpoint_config.target.email_name.clone()))
        // ... or by an address only
        .from(endpoint_config.from_email.clone())
        .reply_to(&data.email[..])
        .subject(format!("[{endpoint_name}] Contact from {name}", endpoint_name=endpoint_config.name, name=data.name))
        .alternative(
            format!("<p>{name} wrote:</p><br><i>{message}</i>\n\n<br><br><p>Reply to <a href=\"mailto:{email}\">{email}</a> or {phone}</p>", 
                name=data.name, 
                message=data.message.replace("\n", "<br>"), 
                phone=phone_contact, 
                email=data.email),
            format!("Message from {name}:\n\n> {message}\n\nReply to <{email}> or {phone}", 
                name=data.name, 
                message=data.message, 
                phone=phone_contact, 
                email=data.email))
        .build()
        .expect("Error while building mail");

    // Connect to a SMTP-Server
    let mut mailer = SmtpClient::new_simple(&config.server.smtp_server).unwrap()
        // Set the name sent during EHLO/HELO, default is `localhost`
        .hello_name(ClientId::Domain("atoav.com".to_string()))
        // Add credentials for authentication
        .credentials(Credentials::new(config.server.smtp_user.to_string(), config.server.smtp_password.to_string()))
        // Enable SMTPUTF8 if the server supports it
        .smtp_utf8(true)
        // Configure expected authentication mechanism
        .authentication_mechanism(Mechanism::Plain)
        // Enable connection reuse
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited).transport();
    
    mailer.send(email.into())
}