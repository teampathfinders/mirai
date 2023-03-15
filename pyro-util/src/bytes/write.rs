use paste::paste;

macro_rules! declare_write_fns {
    ($($ty: ident),+) => {
        paste! {$(
            #[doc = concat!("Writes a little endian [`", stringify!($ty), "'] to the writer")]
            fn [<write_ $ty _le>](&mut self, v: $ty);
            #[doc = concat!("Writes a big endian [`", stringify!($ty), "'] to the writer")]
            fn [<write_ $ty _be>](&mut self, v: $ty);
        )+}
    }
}

pub trait BinaryWriter {
    declare_write_fns!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

    fn write_bool(&mut self, v: bool);
    fn write_u8(&mut self, v: u8);
    fn write_i8(&mut self, v: i8);
    fn write_var_u32(&mut self, v: u32);
    fn write_var_u64(&mut self, v: u64);
    fn write_var_i32(&mut self, v: i32);
    fn write_var_i64(&mut self, v: i64);
    fn write_str(&mut self, v: &str);
}
