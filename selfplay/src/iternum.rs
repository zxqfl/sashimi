use import::*;

#[derive(Serialize, Deserialize, Default)]
pub struct Iternum(usize);

impl Iternum {
    pub fn increment(self) -> Self {
        Iternum(self.0 + 1)
    }

    pub fn name_for(&self, x: &str) -> PathBuf {
        format!("{}_{}.json", x, self.0).into()
    }
}

use std::fmt::{self, Display};
impl Display for Iternum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
