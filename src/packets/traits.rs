use bytes::BytesMut;
use crate::error::VexResult;

pub trait RaknetPacket {
    const ID: u8;
}

pub trait Encodable {
    fn encode(&self) -> BytesMut;
}

pub trait Decodable {
    fn decode(buf: BytesMut) -> VexResult<Self> where Self: Sized;
}

#[macro_export]
macro_rules! decodable {
    ($($args: tt)*) => {
        $crate::__impl_general_packet!($($args)*);
    }
}

#[macro_export]
macro_rules! encodable {
    ($($args: tt)*) => {
        $crate::__impl_general_packet!($($args)*);
        $crate::__impl_builder_packet!($($args)*);
    }
}

#[macro_export]
macro_rules! __impl_general_packet {
    ($id: expr, $vis: vis struct $name: ident $(<$($lt: lifetime),+>)* {
        $($fname: ident : $ftype: ty),*
    }) => {
        paste::paste! {
            #[derive(Default)]
            $vis struct $name $(<$($lt),*>)* {
                $($fname: $ftype),*
            }
            impl $(<$($lt),*>)* $name $(<$($lt),*>)* {
                $(
                    fn $fname(&self) -> &$ftype {
                        &self.$fname
                    }
                    fn [<$fname _mut>](&mut self) -> &mut $ftype {
                        &mut self.$fname
                    }
                )*
            }
            impl $(<$($lt),*>)* $crate::packets::RaknetPacket for $name $(<$($lt),*>)* {
                const ID: u8 = $id;
            }
        }
    };
}

#[macro_export]
macro_rules! __impl_builder_packet {
    ($id: expr, $vis: vis struct $name: ident $(<$($lt: lifetime),+>)* {
        $($fname: ident : $ftype: ty),*
    }) => {
        paste::paste! {
            impl $(<$($lt),*>)* $name $(<$($lt),*>)* {
                fn build() -> [<$name Builder>] $(<$($lt),*>)* {
                    [<$name Builder>]::default()
                }
            }
            #[derive(Default)]
            $vis struct [<$name Builder>] $(<$($lt),*>)* {
                pk: $name $(<$($lt),*>)*
            }
            impl $(<$($lt),*>)* [<$name Builder>] $(<$($lt),*>)* {
                $(
                    fn $fname(mut self, $fname: $ftype) -> Self {
                        self.pk.$fname = $fname;
                        self
                    }
                )*
                fn encode(self) -> bytes::BytesMut {
                    self.pk.encode()
                }
            }
        }
    };
}