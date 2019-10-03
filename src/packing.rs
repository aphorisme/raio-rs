pub use message::*;
pub use pack::*;
pub use types::*;
pub use unpack::*;

// Packing modules:
mod message;
mod pack;
mod types;
mod unpack;

// Explicit import modules:
pub mod error;
pub mod ll;

// further forwarding:
pub use ll::ValueList;
pub use ll::ValueMap;

// Testing
#[cfg(test)]
mod test {
    macro_rules! single_value_tests {
        ($($i:ident : $t:ty => $($e:expr),+);+) => {
            $(mod $i {
                use crate::packing::*;
                use std::convert::TryFrom;

                #[test]
                fn pack_unpack_exact() {
                    for i in vec!($($e),+) {
                        let mut b : Vec<u8> = Vec::new();
                        let x : $t = <$t>::try_from(i).unwrap();
                        b.pack(&x).unwrap();

                        let u =
                            <$t>::unpack_from(&mut b.clone().as_slice()).unwrap();

                        assert_eq!(x, u);
                    }
                }

                #[test]
                fn pack_bytes_exact() {
                    for i in vec!($($e),+) {
                        let mut b : Vec<u8> = Vec::new();
                        let x : $t = <$t>::try_from(i).unwrap();
                        let written = b.pack(&x).unwrap();

                        assert_eq!(b.len(), written);
                    }
                }
            })+
        }
    }

    single_value_tests! {
        i64 : i64 => 42, 134545, 12, -123, 452;

        string : String => "jehajs dhje ", "", "jkaanann";

        f64 : f64 => 4.2, -124.5, 0.0, -12453.12312;

        bool : bool => false, true;

        plus_tiny_int : PlusTinyInt => 127, 0, 12, 42;
        minus_tiny_int : MinusTinyInt => -12, -1, -5;

        int_8  : Int8  => 127, 0, -54, 32;
        int_16 : Int16 => 34, 19771, -12, 0, 5344;
        int_32 : Int32 => 42i32, -421i32, 446i32, 124i32;
        int_64 : Int64 => 193883, -1, 0, 12331, 311;

        float_64 : Float64 => 432.2, -13.4, 13334.0, 123.4222;

        tiny_struct_node : Node =>
            Node {
                node_identity: 42,
                labels: ValueList(vec!("label01".into(), "label02".into(), "label03".into())),
                properties: vec! {
                    ("prop01".to_string(), Value::Boolean(true)),
                    ("prop02".to_string(), Value::Integer(54)),
                    ("prop03".to_string(), Value::UnboundRelationship(UnboundRelationship { rel_identity: 43, type_s: "atype".to_string(), properties: ValueMap::new() }))
                }.into_iter().collect(),
            };

        value : Value =>
            Value::from(423i64),
            Value::from(String::from("HelloWorld")),
            Value::from(String::from("")),
            Value::from(false),
            Value::from(true),
            Value::from(<Option<i64>>::None),
            Value::from(42.42f64),
            Value::from(Node {
                node_identity: 12,
                labels: ValueList(vec!("label01".into(), "label02".into(), "label03".into(), "jkj ejkr".to_string())),
                properties: vec! {
                    ("prop01".to_string(), Value::Boolean(true)),
                    ("prop02".to_string(), Value::Integer(54)),
                    ("prop03".to_string(), Value::UnboundRelationship(UnboundRelationship { rel_identity: 21, type_s: "k".to_string(), properties: ValueMap::new() })),
                    ("another".to_string(), Value::Null)
                }.into_iter().collect(),
            });

        value_map : ValueMap =>
            ValueMap::new(),
            {
                let mut vm = ValueMap::with_capacity(3);
                vm.insert_value("hello", 42);
                vm.insert_value("world", true);
                vm.insert_value("foo", vec!(23i64, 33i64));
                vm
            },
            {
                let mut vm = ValueMap::with_capacity(2);
                vm.insert_value("null", Value::Null);
                vm.insert_value("float", 42.42);
                vm
            };

        value_list: ValueList =>
            ValueList::new(),
            ValueList::from(vec!(32i64, 42i64, 0)),
            ValueList::from(vec!("hello", "world")),
            ValueList::from(vec!(Value::Null, Value::Null))
    }
}
