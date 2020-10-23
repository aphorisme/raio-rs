use packs::*;
use packs::std_structs::StdStruct;

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x01]
/// The `HELLO` request message in the bolt protocol. Can be encoded using PackStream:
pub struct Hello {
   extra: Dictionary<StdStruct>,
}

impl Hello {
   pub fn new(agent_name: &str, version: &str, scheme: &str, principal: &str, credentials: &str) -> Self {
      let mut extra = <Dictionary<StdStruct>>::with_capacity(4);
      extra.add_property("user_agent", format!("{}/{}", agent_name, version));
      extra.add_property("scheme", scheme);
      extra.add_property("principal", principal);
      extra.add_property("credentials", credentials);

      Hello {
         extra,
      }
   }
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x02]
pub struct GoodBye {}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x0F]
pub struct Reset {}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x10]
///  bookmarks::List<String>, tx_timeout::Integer, tx_metadata::Dictionary, mode::String, db:String)
pub struct Run {
   query: String,
   parameters: Dictionary<StdStruct>,
   extra: Dictionary<StdStruct>,
}

impl Run {
   pub fn new(query: &str) -> Run {
      Run {
         query: String::from(query),
         parameters: Dictionary::new(),
         extra: Dictionary::new(),
      }
   }

   pub fn param<V: Into<Value<StdStruct>>>(&mut self, param: &str, value: V){
      self.parameters.add_property(param, value);
   }
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x2F]
pub struct Discard {
   extra: Dictionary<StdStruct>
}

impl Discard {
   pub fn new(n: Option<i64>, qid: Option<i64>) -> Self {
      let mut extra = Dictionary::with_capacity(1);

      if let Some(n) = n {
         extra.add_property("n", n);
      }

      if let Some(qid) = qid {
         extra.add_property("qid", qid);
      }

      Discard { extra }
   }
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x3F]
pub struct Pull {
   extra: Dictionary<StdStruct>
}

impl Pull {
   pub fn new(n: Option<i64>, qid: Option<i64>) -> Self {
      let mut extra = Dictionary::with_capacity(1);

      if let Some(n) = n {
         extra.add_property("n", n);
      }

      if let Some(qid) = qid {
         extra.add_property("qid", qid);
      }

      Pull { extra }
   }

   pub fn all(qid: Option<i64>) -> Self {
      Self::new(Some(-1), qid)
   }

   pub fn all_from_last() -> Self {
      Self::new(Some(-1), Some(-1))
   }
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x11]
// bookmarks::List<String>, tx_timeout::Integer, tx_metadata::Dictionary, mode::String, db::String
pub struct Begin {
   extra: Dictionary<StdStruct>
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x12]
pub struct Commit {}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x13]
pub struct RollBack {}