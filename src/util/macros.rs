#[macro_export]
macro_rules! generate_getters {
    ($vis: vis struct $name: ident $(<$($lt: lifetime),+>)* {
        $($fname: ident : $ftype: ty),*
    }) => {
        paste::paste! {
            #[derive(Default)]
            pub struct $name $(<$($lt),+>)* {
                $($fname: $ftype),*
            }
            impl $(<$($lt),+>)* $name $(<$($lt),+>)* {
                $(
                    #[inline]
                    pub fn $fname(&self) -> &$ftype {
                        &self.$fname
                    }
                    #[inline]
                    pub fn [<$fname _mut>](&mut self) -> &mut $ftype {
                        &mut self.$fname
                    }
                )*
            }
        }
    }
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
            $crate::generate_getters!($vis struct $name $(<$($lt),+>)* {
                $($fname: $ftype),*
            });
            impl $(<$($lt),*>)* $crate::raknet::packets::RaknetPacket for $name $(<$($lt),*>)* {
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
                pub fn build() -> [<$name Builder>] $(<$($lt),*>)* {
                    [<$name Builder>]::default()
                }
            }
            #[derive(Default)]
            $vis struct [<$name Builder>] $(<$($lt),*>)* {
                pk: $name $(<$($lt),*>)*
            }
            impl $(<$($lt),*>)* [<$name Builder>] $(<$($lt),*>)* {
                $(
                    pub fn $fname(mut self, $fname: $ftype) -> Self {
                        self.pk.$fname = $fname;
                        self
                    }
                )*
                pub fn encode(self) -> bytes::BytesMut {
                    self.pk.encode()
                }
            }
        }
    };
}
