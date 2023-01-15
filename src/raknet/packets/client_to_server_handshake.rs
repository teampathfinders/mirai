#[derive(Debug)]
pub struct clienttoserverhandshake {
    pub jwt_data	: i32,
    // This packet has no data.

}


impl clienttoserverhandshake {
    pub const ID: u8 = 0x04;
}
