use crate::types::{read_be_u64, Packet};
use std::fmt::{Debug, Formatter};

pub struct MOTD {
    pub client_send_time_be: u64,
    pub client_guid_be: u64,
}
impl Debug for MOTD {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ client_send_time_be: {:?}, client_guid_be: {:?} }}", self.client_send_time_be, self.client_guid_be)
    }
}

impl Packet for MOTD {
    fn serialize(&self) -> Vec<u8> {
        let result: Vec<String> = vec![];

        result.join(";").into()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 32 {
            return Err("Invalid data length".to_string());
        }
        let client_send_time_be = read_be_u64(&data);
        let client_guid_be = read_be_u64(&data[24..]);

        Ok(MOTD {
            client_send_time_be,
            client_guid_be,
        })
    }
}
