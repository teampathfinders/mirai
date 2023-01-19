#[derive(Debug)]
pub enum AckRecord {
    Single(u32),
    Range(u32, u32)
}

#[derive(Debug)]
pub struct Ack {
    pub records: Vec<AckRecord>
}

impl Ack {
    pub const ID: u8 = 0xc0;
}

#[derive(Debug)]
pub struct Nack {
    pub records: Vec<AckRecord>
}

impl Nack {
    pub const ID: u8 = 0xa0;
}