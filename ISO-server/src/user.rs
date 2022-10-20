use openssl::rand;
use phonenumber::*;
use crate::{post::Post, CONFIG, data::{db_clone, db_mut}};
use serde_json::json;
use ::serde::{Deserialize, Serialize};
use reqwest::Client;

const POSSIBLE_CODE_CHARS: &'static [char] = &[
    '2', '3', '4', '6', '7', '9', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'P', 'A', 'D', 'F', 'G', 'H',
    'X',
];

/// User data
#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub uuid: String,
    // Used to authenticate user
    token: String,
    /// E.164 phone number
    phone_number: String,
    current_location: (f64, f64),
    karma: i32,
    posts: Vec<String>,
    verified: String,
}


impl User {
    /// Create a new user
    pub fn new(uuid: String, phone_number: String) -> Result<User, String> {
        Ok(User {
            uuid,
            token: User::generate_token(),
            phone_number,
            current_location: (0.0, 0.0),
            karma: 0,
            posts: Vec::new(),
            verified: String::new(),
        })
    }

    pub async fn start_verification(phone_number: String, country: String) -> Result<String, String> {
        let id: Option<phonenumber::country::Id> = country.parse().ok();

        let number = phonenumber::parse(id, phone_number);

        if number.is_err() {
            return Err("Error parsing phone number".to_string());
        }

        let number = number.unwrap();

	    let valid  = phonenumber::is_valid(&number);

        if !valid {
            return Err("Invalid phone number".to_string());
        }

        let phone_number = number.format().mode(Mode::E164).to_string();

        let lock = CONFIG.lock().await;

        let client = reqwest::Client::new();

        // Set custom random 6 digit code
        
        let res = client.post(format!("https://verify.twilio.com/v2/Services/{}/Verifications", lock.twilio_service))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(lock.twilio_sid.clone(), Some(lock.twilio_token.clone()))
            .form(&[("To", phone_number.clone()), ("Channel", "sms".to_string())])
            .send()
            .await;

        if res.is_err() {
            return Err("Error sending verification request".to_string());
        }

        let res = res.unwrap();

        let text = res.text().await.unwrap();
        // parse as json
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();

        println!("{:?}", json);
        let code = json["url"].as_str();

        if code.is_none() {
            return Err("Error getting verification status, please wait 10 minutes.".to_string());
        }

        let code = code.unwrap();

        let mut db = db_mut().await;

        let user = db.get_user_by_number(&phone_number);

        if user.is_ok() {
            let uuid = user.as_ref().unwrap().uuid.clone();
            user.unwrap().set_verification_code(code.to_string());

            return Ok(uuid);
        }

        let user = User::new(uuid::Uuid::new_v4().to_string(), phone_number);

        if user.is_err() {
            return Err("Error creating user".to_string());
        }

        let mut user = user.unwrap();

        user.set_verification_code(code.to_string());

        let _ = db.add_user(user.clone());

        drop(db);

        Ok(user.uuid)
    }

    pub async fn check_verification(&mut self, code: String) -> Result<User, String> {
        let client = reqwest::Client::new();
        let lock = CONFIG.lock().await;

        // Set custom random 6 digit code
        
        let res = client.post(format!("https://verify.twilio.com/v2/Services/{}/VerificationCheck", lock.twilio_service))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(lock.twilio_sid.clone(), Some(lock.twilio_token.clone()))
            .form(&[("To", self.phone_number.clone()), ("Code", code)])
            .send()
            .await;

        if res.is_err() {
            return Err("Error sending verification check request".to_string());
        }

        let res = res.unwrap();

        let text = res.text().await.unwrap();

        // parse as json
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();

        println!("{:?}", json);

        if json["status"].as_str().unwrap() != "approved" {
            return Err("Invalid verification code".to_string());
        }

        self.verified = "true".to_string();

        Ok(self.clone())
    }


    pub fn generate_token() -> String {
        let mut code = String::new();

        for _ in 0..256 {
            let mut buf = [0u8; 1];
            rand::rand_bytes(&mut buf).unwrap();

            code.push(POSSIBLE_CODE_CHARS[buf[0] as usize % POSSIBLE_CODE_CHARS.len()]);
        }

        code
    }

    pub fn generate_code() -> String {
        let mut code = String::new();

        for _ in 0..6 {
            let mut buf = [0u8; 1];
            rand::rand_bytes(&mut buf).unwrap();

            code.push(POSSIBLE_CODE_CHARS[buf[0] as usize % POSSIBLE_CODE_CHARS.len()]);
        }

        code
    }

    pub fn set_verification_code(&mut self, code: String) -> &mut Self {
        self.verified = code;

        self
    }

    pub fn get_token(&self) -> String {
        self.token.clone()
    }
    
    pub fn get_phone_number(&self) -> String {
        self.phone_number.clone()
    }

    pub fn add_post(&mut self, post: String) -> &mut Self {
        self.posts.push(post);

        self
    }

    pub fn add_claimed_post(&mut self, post: String) -> &mut Self {
        self.posts.push(post);

        self
    }
}