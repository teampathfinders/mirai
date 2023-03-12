pub trait BufMut {
    fn write_bool(&mut self, value: bool);

    fn write_u8(&mut self, value: u8);
    fn write_u16(&mut self, value: u16);
    fn write_u32(&mut self, value: u32);
    fn write_u64(&mut self, value: u64);
    fn write_u128(&mut self, value: u128);

    fn write_i8(&mut self, value: i8);
    fn write_i16(&mut self, value: i16);
    fn write_i32(&mut self, value: i32);
    fn write_i64(&mut self, value: i64);
    fn write_i128(&mut self, value: i128);

    fn write_u16_le(&mut self, value: u16);
    fn write_u32_le(&mut self, value: u32);
    fn write_u64_le(&mut self, value: u64);
    fn write_u128_le(&mut self, value: u128);

    fn write_i16_le(&mut self, value: i16);
    fn write_i32_le(&mut self, value: i32);
    fn write_i64_le(&mut self, value: i64);
    fn write_i128_le(&mut self, value: i128);

    fn write_f32(&mut self, value: f32);
    fn write_f32_le(&mut self, value: f32);
    fn write_f64(&mut self, value: f64);
    fn write_f64_le(&mut self, value: f64);
}
