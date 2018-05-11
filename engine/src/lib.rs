#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate float_ord;
extern crate smallvec;
extern crate shakmaty;
extern crate chess;
extern crate mcts;

mod import;
pub mod model;
pub mod search;
pub mod evaluation;
pub mod policy_features;
pub mod features;
pub mod features_common;
pub mod state;
