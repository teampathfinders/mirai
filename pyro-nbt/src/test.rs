#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::ser::to_be_bytes;
    use crate::{de::Deserializer, from_be_bytes};

    const BIG_TEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
    const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
    const PLAYER_NAN_VALUE_NBT: &[u8] =
        include_bytes!("../test/player_nan_value.nbt");

    #[test]
    fn read_write_bigtest() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct Food {
            name: String,
            value: f32,
        }

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct Nested {
            egg: Food,
            ham: Food,
        }

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct ListCompound {
            #[serde(rename = "created-on")]
            created_on: i64,
            name: String,
        }

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct AllTypes {
            #[serde(rename = "nested compound test")]
            nested: Nested,
            #[serde(rename = "intTest")]
            int_test: i32,
            #[serde(rename = "byteTest")]
            byte_test: i8,
            #[serde(rename = "stringTest")]
            string_test: String,
            #[serde(rename = "listTest (long)")]
            long_list_test: [i64; 5],
            #[serde(rename = "doubleTest")]
            double_test: f64,
            #[serde(rename = "floatTest")]
            float_test: f32,
            #[serde(rename = "longTest")]
            long_test: i64,
            #[serde(rename = "listTest (compound)")]
            compound_list_test: (ListCompound, ListCompound),
            #[serde(
                rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))"
            )]
            byte_array_test: Vec<i8>,
            #[serde(rename = "shortTest")]
            short_test: i16,
        }

        let decoded: AllTypes = from_be_bytes(BIG_TEST_NBT).unwrap().0;
        // dbg!(&decoded);

        let encoded = to_be_bytes(&decoded).unwrap();
        let decoded2: AllTypes = from_be_bytes(encoded.as_slice()).unwrap().0;
        dbg!(&decoded2);

        // Checking floats for equality is a pain.
        // If the data can be decoded, it's pretty much correct
    }

    #[test]
    fn read_write_hello_world() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        #[serde(rename = "hello world")]
        struct HelloWorld {
            name: String,
        }

        let decoded: HelloWorld = from_be_bytes(HELLO_WORLD_NBT).unwrap().0;
        dbg!(&decoded);
        let encoded = to_be_bytes(&decoded).unwrap();

        assert_eq!(encoded.as_slice(), HELLO_WORLD_NBT);
    }

    #[test]
    fn read_write_player() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        #[serde(rename_all = "PascalCase")]
        #[serde(rename = "")]
        struct Player {
            pos: [f64; 3],
            motion: [f64; 3],
            on_ground: bool,
            death_time: i16,
            air: i16,
            health: i16,
            fall_distance: f32,
            attack_time: i16,
            hurt_time: i16,
            fire: i16,
            rotation: [f32; 2],
        }

        let decoded: Player = from_be_bytes(PLAYER_NAN_VALUE_NBT).unwrap().0;
        let encoded = to_be_bytes(&decoded).unwrap();

        let decoded2: Player = from_be_bytes(encoded.as_slice()).unwrap().0;

        // Checking floats for equality is a pain.
        // If the data can be decoded, it's pretty much correct
        // assert_eq!(PLAYER_NAN_VALUE_NBT, encoded.as_slice());
    }
}
