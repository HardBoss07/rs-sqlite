use crate::consts::{
    EMAIL_OFFSET, EMAIL_SIZE, ID_OFFSET, ROW_SIZE, USERNAME_OFFSET, USERNAME_SIZE,
};

#[derive(Debug, Clone)]
pub struct Row {
    pub id: u32,
    pub username: [u8; USERNAME_SIZE],
    pub email: [u8; EMAIL_SIZE],
}

impl Row {
    pub fn serialize(&self, destination: &mut [u8]) {
        destination[ID_OFFSET..USERNAME_OFFSET].copy_from_slice(&self.id.to_ne_bytes());
        destination[USERNAME_OFFSET..EMAIL_OFFSET].copy_from_slice(&self.username);
        destination[EMAIL_OFFSET..ROW_SIZE].copy_from_slice(&self.email);
    }

    pub fn deserialize(source: &[u8]) -> Self {
        let mut id_bytes = [0u8; 4];
        id_bytes.copy_from_slice(&source[ID_OFFSET..USERNAME_OFFSET]);
        let id = u32::from_ne_bytes(id_bytes);

        let mut username = [0u8; USERNAME_SIZE];
        username.copy_from_slice(&source[USERNAME_OFFSET..EMAIL_OFFSET]);

        let mut email = [0u8; EMAIL_SIZE];
        email.copy_from_slice(&source[EMAIL_OFFSET..ROW_SIZE]);

        Self {
            id,
            username,
            email,
        }
    }

    pub fn print(&self) {
        let valid_username = self.username.split(|&b| b == 0).next().unwrap_or(&[]);
        let valid_email = self.email.split(|&b| b == 0).next().unwrap_or(&[]);

        let username_str = std::str::from_utf8(valid_username).unwrap_or("Error parsing username.");
        let email_str = std::str::from_utf8(valid_email).unwrap_or("Error parsing Email.");

        println!("({}, {}, {})", self.id, username_str, email_str);
    }
}
