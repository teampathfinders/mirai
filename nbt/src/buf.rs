pub trait Buf {
    fn read_bool(&mut self) -> Option<bool>;

    fn read_u8(&mut self) -> Option<u8>;
    fn read_u16(&mut self) -> Option<u16>;
    fn read_u32(&mut self) -> Option<u32>;
    fn read_u64(&mut self) -> Option<u64>;
    fn read_u128(&mut self) -> Option<u128>;

    fn read_i8(&mut self) -> Option<i8>;
    fn read_i16(&mut self) -> Option<i16>;
    fn read_i32(&mut self) -> Option<i32>;
    fn read_i64(&mut self) -> Option<i64>;
    fn read_i128(&mut self) -> Option<i128>;

    fn read_u16_le(&mut self) -> Option<u16>;
    fn read_u32_le(&mut self) -> Option<u32>;
    fn read_u64_le(&mut self) -> Option<u64>;
    fn read_u128_le(&mut self) -> Option<u128>;

    fn read_i16_le(&mut self) -> Option<i16>;
    fn read_i32_le(&mut self) -> Option<i32>;
    fn read_i64_le(&mut self) -> Option<i64>;
    fn read_i128_le(&mut self) -> Option<i128>;

    fn read_f32(&mut self) -> Option<f32>;
    fn read_f32_le(&mut self) -> Option<f32>;
    fn read_f64(&mut self) -> Option<f64>;
    fn read_f64_le(&mut self) -> Option<f64>;
}
