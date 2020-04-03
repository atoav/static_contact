//! The [FormData] struct stores the deserialized JSON body sent by the endpoints. After deserialization the fields are checked for conformity and return an Error if something is off.
//! As for now the fields of the struct are fixed, but maybe in the Future FormData could be a trait that could be implemented on custom structs, that could be derived from the individual endpoint configurations

use check_if_email_exists::email_exists;
use serde::Deserialize;
use crate::config::EndpointConfig;
use crate::GenResult;

/// This stores the Data received via JSON POST request.
#[derive(Deserialize)]
pub struct FormData {
    pub name:          String,
    pub email:         String,
    pub phone:         String,
    pub message:       String,
    pub identifier:    String
}


impl FormData{
    /// Checks the length of [FormData] fields for certain minimum or maximum lengths. Some of these limits can be defined in the `config.toml`
    pub fn check_length(&mut self, endpoint_config: &EndpointConfig) -> GenResult<()> {
        let mut error_message = "".to_string();

        if self.name.len() > endpoint_config.max_name_length{
            error_message += format!("Field \"name\" was too long: {} characters ({} is maximum)\n", self.name.len(), endpoint_config.max_name_length).as_str()
        }
        if self.email.len() > 254{
            error_message += format!("Field \"email\" was too long: {} characters ({} is maximum)\n", self.email.len(), 254).as_str()
        }else if self.email.len() < 3{
            error_message += format!("Field \"email\" was too short: {} characters ({} is minimum)\n", self.email.len(), 3).as_str()
        }
        if self.phone.len() > 32{
            error_message += format!("Field \"phone\" was too long: {} characters ({} is maximum)\n", self.phone.len(), 32).as_str()
        }else if self.phone.len() != 0 && self.phone.len() < 4{
            error_message += format!("Field \"phone\" was too short: {} characters ({} is minimum)\n", self.phone.len(), 4).as_str()
        }
        if self.message.len() > endpoint_config.max_message_length{
            error_message += format!("Field \"message\" was too long: {} characters ({} is maximum)", self.message.len(), endpoint_config.max_message_length).as_str()
        }

        if error_message != "".to_string(){
            error_message = error_message.trim().to_string();
            bail!(error_message)
        }

        Ok(())
    }

    /// Check for the existence of the user-provided email adress 
    pub async fn check_existence(&mut self, _endpoint_config: &EndpointConfig) -> GenResult<()> {
        let mut error_message = "".to_string();
        println!("Checking existence of email adress");
        let checked = email_exists(&self.email, "dh@atoav.com").await;

        // Check email syntax
        match checked.syntax{
            Ok(s) => {
                if !s.valid_format {
                    error_message += format!("The provided email adress \"{}\" seems to be invalid.\n", self.email).as_str();
                }
            },
            Err(e) => error_message += format!("Error while checking email syntax: {:?}", e).as_str()
        }

        // Check email existence via SMTP
        match checked.smtp{
            Ok(s) => {
                if s.has_full_inbox {
                    error_message += format!("The provided email adress \"{}\" has a full inbox and can't receive new emails.\n", self.email).as_str();
                }
                if !s.is_deliverable {
                    error_message += format!("Email can't be delivered to adress \"{}\".\n", self.email).as_str();
                }
                if s.is_disabled {
                    error_message += format!("Your Mailserver provider blocked/disabled the email adress \"{}\".\n", self.email).as_str();
                }
            },
            Err(e) => {
                if format!("{:?}", e).contains("Skipped"){
                    println!("Skipped checking SMTP validity")
                }else{
                    eprintln!("Error while checking SMTP validity: {:?}", e)
                }
            }
        }

        // Check if the mail is disposable
        match checked.misc{
            Ok(s) => {
                if s.is_disposable {
                    error_message += format!("The provided email adress \"{}\" seems to be disposable/invalid adress.\n", self.email).as_str();
                }
            },
            Err(e) => if format!("{:?}", e).contains("Skipped"){
                    println!("Skipped checking for disposable email")
                }else{
                    eprintln!("Error while checking for disposable email: {:?}", e)
                }
        }

        // Output all errors
        if error_message != "".to_string(){
            error_message = error_message.trim().to_string();
            bail!( error_message)
        }
        
        Ok(())
    }
}