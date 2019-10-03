use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltRead, BoltWrite, Signature, ValueList, ValueMap, WResult};
use crate::packing::types::{TinyStructBody, Value};
use crate::packing::{Packable, Unpackable};

macro_rules! count_u8 {
    ($t:ident) => { 1u8 };
    ($t:ident , $($tails:ident),*) => { 1u8 + count_u8!($($tails),*) }
}

/// This macro allows more then 15 fields, which leads to wrong encoding/decoding.
/// Whenever used, the maximum number of fields specified must not exceed 15.
macro_rules! bolt_tiny_struct {
    ( $name:ident : $sig:expr => { $( $field:ident : $fty:ty ),+ } ) => {
        # [derive(Debug, PartialEq, Clone)]
        pub struct $name {
            $( pub $field: $fty, )+
        }

        impl TinyStructBody for $name {
            const FIELDS: u8 = count_u8!($($field),+);
            const SIGNATURE: Signature = $sig;

            fn write_body_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError> {
                let written = 0
                   $(+ self.$field.pack_to(buf)?)+;
                Ok(written)
            }

            fn read_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
                Ok ( $name {
                    $( $field : <$fty>::unpack_from(buf)?, )+
                })
            }
        }
    }
}

bolt_tiny_struct!(
    Node : Signature::Node => {
        node_identity : i64,
        labels: ValueList<String>,
        properties: ValueMap<Value>
    });

bolt_tiny_struct!(
    UnboundRelationship : Signature::UnboundRelationship => {
        rel_identity: i64,
        type_s: String,
        properties: ValueMap<Value>
    });

bolt_tiny_struct!(
     Relationship : Signature::Relationship => {
        rel_identity: i64,
        start_node_identity: i64,
        end_node_identity: i64,
        type_s: String,
        properties: ValueMap<Value>
    });

bolt_tiny_struct!(
    Path : Signature::Path => {
        nodes: ValueList<Node>,
        relationships: ValueList<UnboundRelationship>,
        sequence: ValueList<i64>
    });

// message structs:
bolt_tiny_struct!(
    Init : Signature::Init => {
        client_name: String,
        auth_token: ValueMap<Value>
    });

bolt_tiny_struct!(
    Hello : Signature::Init => {
        auth_token: ValueMap<Value>
    });

bolt_tiny_struct!(
    Run : Signature::Run => {
        statement: String,
        parameters: ValueMap<Value>,
        metadata: ValueMap<Value>
    });

impl Run {
    pub fn statement(raw: &str) -> Run {
        Run {
            statement: raw.to_string(),
            parameters: <ValueMap<Value>>::new(),
            metadata: <ValueMap>::new(),
        }
    }

    /// Provides a convenient way to add parameters to the `Run` structure:
    /// ```
    /// use raio::packing::{Run, ValueMap};
    ///
    /// let mut run_struct =
    ///     Run::statement("MATCH (n) WHERE n.name == {name} AND n.age == {age}");
    ///
    /// run_struct.param("name", "John Doe")
    ///           .param("age", 42);
    ///
    /// let mut params = <ValueMap>::with_capacity(2);
    /// params.insert_value("name", "John Doe");
    /// params.insert_value("age", 42);
    ///
    /// assert_eq!(run_struct.parameters, params);
    /// ```
    /// This does not run any syntactic or semantic checks. It is totally valid to add a parameter
    /// which does not exist in the statement.
    pub fn param<V>(&mut self, name: &str, value: V) -> &mut Self
    where
        Value: From<V>,
    {
        self.parameters.insert_value(name, value);
        self
    }

    pub fn meta<V>(&mut self, name: &str, value: V) -> &mut Self
    where
        Value: From<V>,
    {
        self.metadata.insert_value(name, value);
        self
    }
}

bolt_tiny_struct!(
    Record : Signature::Record => {
        fields : ValueList<Value>
    });

bolt_tiny_struct!(
    Success : Signature::Success => {
        metadata : ValueMap<Value>
    });

bolt_tiny_struct!(
    Failure : Signature::Failure => {
        metadata : ValueMap<Value>
    });

macro_rules! bolt_empty_struct {
    ($name:ident : $sig:expr) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $name {}
        impl TinyStructBody for $name {
            const FIELDS: u8 = 0;
            const SIGNATURE: Signature = $sig;
            fn write_body_to<T: BoltWrite>(&self, _: &mut T) -> Result<usize, PackError> {
                Ok(0)
            }
            fn read_body_from<T: BoltRead>(_: &mut T) -> Result<Self, UnpackError> {
                Ok($name {})
            }
        }
    };
}

bolt_empty_struct!(DiscardAll: Signature::DiscardAll);
bolt_empty_struct!(PullAll: Signature::PullAll);
bolt_empty_struct!(AckFailure: Signature::AckFailure);
bolt_empty_struct!(Reset: Signature::Reset);
bolt_empty_struct!(Ignored: Signature::Ignored);

#[cfg(test)]
mod test {
    use crate::packing::{Init, Packable, PullAll, Unpackable, ValueMap};

    #[test]
    fn init_struct_hex() {
        let mut auth_options = ValueMap::with_capacity(3);
        auth_options.insert_value("scheme", "basic");
        auth_options.insert_value("credentials", "secret");
        auth_options.insert_value("principal", "neo4j");

        let init_struct = Init {
            client_name: "MyClient/1.0".to_string(),
            auth_token: auth_options,
        };

        let control_bytes: Vec<u8> = vec![
            0xB2, 0x01, 0x8C, 0x4D, 0x79, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x2F, 0x31, 0x2E,
            0x30, 0xA3, // TinyMap, 3 items
            0x8B, 0x63, 0x72, 0x65, 0x64, 0x65, 0x6E, 0x74, 0x69, 0x61, 0x6C, 0x73, // Key
            0x86, 0x73, 0x65, 0x63, 0x72, 0x65, 0x74, // Value
            0x86, 0x73, 0x63, 0x68, 0x65, 0x6D, 0x65, // Key
            0x85, 0x62, 0x61, 0x73, 0x69, 0x63, // Value
            0x89, 0x70, 0x72, 0x69, 0x6E, 0x63, 0x69, 0x70, 0x61, 0x6C, // Key
            0x85, 0x6E, 0x65, 0x6F, 0x34, 0x6A, // Value
        ];

        let control_init = Init::unpack_from(&mut control_bytes.as_slice())
            .expect("Cannot unpack init from control bytes");

        assert_eq!(init_struct, control_init);
    }

    #[test]
    fn pull_all_hex() {
        let control_bytes = vec![0xB0, 0x3F];

        let mut pull_all_bytes: Vec<u8> = Vec::new();

        PullAll {}
            .pack_to(&mut pull_all_bytes)
            .expect("Cannot pack PullAll to bytes.");

        assert_eq!(pull_all_bytes, control_bytes);

        let control_pull_all =
            PullAll::unpack_from(&mut pull_all_bytes.as_slice()).expect("Cannot unpack PullAll.");

        assert_eq!(PullAll {}, control_pull_all)
    }
}
