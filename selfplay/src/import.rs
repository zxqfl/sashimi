pub use std::path::PathBuf;
pub use std::io::{Write, BufWriter};
pub use std::fs::File;
pub use playgame::{CycleResult};
pub use self_play_state::SelfPlayState;
pub use config::Config;
pub use rayon::prelude::*;
pub use iternum::Iternum;
pub use chess::{BoardStatus, Color};

pub use mcts::GameState;
pub use engine::state::{Player, State, Move};
pub use engine::features::{self, FeatureVec};
pub use engine::policy_features;
pub use engine::model::Model;
pub use engine::search::Search;

pub extern crate serde_json;
pub extern crate slurp;
pub extern crate rand;
pub use rand::Rng;

pub type Outcome = Option<Player>;
