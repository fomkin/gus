#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub signingkey: Option<String>
}

impl User {
    pub fn change(&mut self, name: Option<String>, email: Option<String>, signingkey: Option<String>) {
        if let Some(name) = name {
            self.name = name
        }
        if let Some(email) = email {
            self.email = email
        }
        if !signingkey.is_none() {
            self.signingkey = signingkey
        }
    }

    pub fn to_cmd(&self) -> Vec<Vec<String>> {
        vec![
            vec!["user.name".to_string(), self.name.to_string()],
            vec!["user.email".to_string(), self.email.to_string()],
            match self.signingkey.clone() {
                Some(signingkey) => vec!["user.signingkey".to_string(), signingkey],
                None => vec!["--unset".to_string(), "user.signingkey".to_string()]
            }
        ]
    }
}

