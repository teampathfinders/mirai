use mirai_macros::*;

#[derive(BedrockCodec)]
struct SerTest {
    #[le]
    hello: i32,
    #[nbt]
    nbt_data: u8
}

#[test]
fn ser_codec() {

}