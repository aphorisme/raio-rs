pub use bolt_read::*;
pub use bolt_write::*;
pub use byte_op::*;
pub use marker::*;
pub use signature::*;
pub use value_list::*;
pub use value_map::*;

mod bolt_read;
mod bolt_write;
mod byte_op;
mod marker;
mod signature;
mod value_list;
mod value_map;

// Tests

#[cfg(test)]
mod test {
    macro_rules! write_read_bolt_exact {
        ($($t:ident => $($e:expr),+);+) => {

            $(
            #[allow(non_snake_case)]
            mod $t {
                use crate::packing::ll::*;

                #[test]
                fn write_read_bolt_exact() {
                    for i in vec!($($e),+) {

                        // write:
                        let mut data : Vec<u8> = Vec::new();
                        data.bolt_write(i.clone()).unwrap();

                        // read:
                        let t_read = data.as_slice().bolt_read::<$t>().unwrap();

                        // compare:
                        assert_eq!(i, t_read);
                    }
                }
            })+
        }
    }

    write_read_bolt_exact! {
        u8 => 42, 13, 45;
        u16 => 32, 1837, 48727;
        u32 => 435, 1298354812, 123127877;
        i8 => -34, -42, -100;
        i16 => -453, 43, 7817;
        i32 => -89, 183717, -421122;
        i64 => 1833771, -83787171, -122222;
        String => String::from("hello world☂"), String::from("wjkehwkehkah jkahjkfawioeioa e02283z891z kjhkjakjziu 2q");
        MarkerByte => MarkerByte::Null, MarkerByte::TinyString, MarkerByte::Struct8, MarkerByte::MinusTinyInt, MarkerByte::PlusTinyInt, MarkerByte::Map16;
        TinySizeMarker => TinySizeMarker { tiny_size: 14, marker: MarkerByte::TinyString }, TinySizeMarker { tiny_size: 0, marker: MarkerByte::Null };
        Signature => Signature::Node, Signature::Init, Signature::Path, Signature::Relationship, Signature::UnboundRelationship, Signature::DiscardAll
    }
}
