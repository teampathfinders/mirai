pub trait Buf {
    type Error;

    fn read_u8(&mut self) -> Result<u8, Self::Error>;
    fn read_u16(&mut self) -> Result<u16, Self::Error>;
    fn read_u32(&mut self) -> Result<u32, Self::Error>;
    fn read_u64(&mut self) -> Result<u64, Self::Error>;
    fn read_u128(&mut self) -> Result<u128, Self::Error>;

    fn read_i8(&mut self) -> Result<i8, Self::Error>;
    fn read_i16(&mut self) -> Result<i16, Self::Error>;
    fn read_i32(&mut self) -> Result<i32, Self::Error>;
    fn read_i64(&mut self) -> Result<i64, Self::Error>;
    fn read_i128(&mut self) -> Result<i128, Self::Error>;

    fn read_u8_le(&mut self) -> Result<u8, Self::Error>;
    fn read_u16_le(&mut self) -> Result<u16, Self::Error>;
    fn read_u32_le(&mut self) -> Result<u32, Self::Error>;
    fn read_u64_le(&mut self) -> Result<u64, Self::Error>;
    fn read_u128_le(&mut self) -> Result<u128, Self::Error>;

    fn read_i8_le(&mut self) -> Result<i8, Self::Error>;
    fn read_i16_le(&mut self) -> Result<i16, Self::Error>;
    fn read_i32_le(&mut self) -> Result<i32, Self::Error>;
    fn read_i64_le(&mut self) -> Result<i64, Self::Error>;
    fn read_i128_le(&mut self) -> Result<i128, Self::Error>;
}