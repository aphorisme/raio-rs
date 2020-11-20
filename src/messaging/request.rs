use packs::std_structs::StdStruct;
use packs::*;
use crate::messaging::commit_prepare::CommitPrepare;
use crate::messaging::query::{Query, query_pack_flat};

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x01]
/// The `HELLO` request message in the bolt protocol. It always has a preset of options set.
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

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x02]
pub struct GoodBye {}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x0F]
pub struct Reset {}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x10]
/// The `RUN` request in the context of a transaction.
pub struct RunInTx {
   query: String,
   parameters: Dictionary<StdStruct>,
   // meant to be empty, but a placeholder for the pack macro:
   extra: Dictionary<StdStruct>,
}

impl RunInTx {
   pub fn new(query: String, parameters: Dictionary<StdStruct>) -> Self {
      RunInTx {
         query,
         parameters,
         extra: Dictionary::new(),
      }
   }
}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x10]
/// The `RUN` request in the context of an Auto-Commit. It therefore carries the
/// `CommitPrepare`.
pub struct Run<'a> {
   #[pack(query_pack_flat)]
   #[fields = 2]
   pub query: &'a Query,
   pub extra: CommitPrepare,
}

impl<'a> Run<'a> {
   pub fn new(query: &'a Query) -> Self {
      Run {
         query,
         extra: CommitPrepare::new(),
      }
   }

   pub fn commit_prepare(&mut self) -> &mut CommitPrepare {
      &mut self.extra
   }
}


#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x2F]
pub struct Discard {
   extra: Dictionary<StdStruct>
}

impl Discard {
   pub fn new(n: Amount, qid: Qid) -> Self {
      let mut extra = Dictionary::with_capacity(2);

      if let Amount::Many(n) = n {
         extra.add_property("n", n);
      } else {
         extra.add_property("n", -1);
      }

      if let Qid::Exact(qid) = qid {
         extra.add_property("qid", qid);
      } else {
         extra.add_property("qid", -1);
      }

      Discard { extra }
   }
}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x3F]
pub struct Pull {
   pub extra: Dictionary<StdStruct>
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Amount {
   Many(i64),
   All,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Qid {
   Exact(i64),
   Last,
}

impl Pull {
   /// Creates a `PULL` request with the provided `n` and `qid`:
   /// ```
   /// # use raio::messaging::request::{Pull, Amount, Qid};
   /// let pull = Pull::new(Amount::All, Qid::Last);
   ///
   /// assert_eq!(pull.extra.get_property_typed("n"), Some(&-1i64));
   /// assert_eq!(pull.extra.get_property_typed("qid"), Some(&-1i64));
   /// ```
   pub fn new(n: Amount, qid: Qid) -> Self {
      let mut extra = Dictionary::with_capacity(2);

      if let Amount::Many(n) = n {
         extra.add_property("n", n);
      } else {
         extra.add_property("n", -1);
      }

      if let Qid::Exact(qid) = qid {
         extra.add_property("qid", qid);
      } else {
         extra.add_property("qid", -1);
      }

      Pull { extra }
   }

   pub fn all(qid: i64) -> Self {
      Self::new(Amount::All, Qid::Exact(qid))
   }

   pub fn all_from_last() -> Self {
      Self::new(Amount::All, Qid::Last)
   }
}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x11]
pub struct Begin {
   extra: CommitPrepare,
}

impl Begin {
   pub fn new(preparation: CommitPrepare) -> Self {
      Begin {
         extra: preparation,
      }
   }

   pub fn commit_prepare(&mut self) -> &mut CommitPrepare {
      &mut self.extra
   }
}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x12]
pub struct Commit {}

#[derive(Debug, Clone, PartialEq, Pack)]
#[tag = 0x13]
pub struct RollBack {}