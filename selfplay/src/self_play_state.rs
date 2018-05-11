use import::*;
use playgame;

const MODEL: &'static str = "model";
const STATE: &'static str = "state";

#[derive(Serialize, Deserialize, Default)]
pub struct SelfPlayState {
    past_results: Vec<CycleResult>,
    iternum: Iternum,
}

impl SelfPlayState {
    pub fn step(self, config: &Config) -> Self {
        let Self { mut past_results, iternum } = self;
        info!("begin step {}", iternum);
        let model_file_str =
            slurp::read_all_to_string(iternum.name_for(MODEL))
            .expect("opening model file (if you want to start from a zero model, \
                     create a blank `model` file)");
        let model = if model_file_str.is_empty() {
            info!("model file is empty, using blank model");
            Model::zero()
        } else {
            serde_json::from_str(&model_file_str).unwrap()
        };
        let cycle_result = playgame::play_cycle(config, &model, &iternum);
        info!("finish step {}", iternum);
        past_results.push(cycle_result);
        let iternum = iternum.increment();
        Self { past_results, iternum }
    }

    pub fn loop_(config: &Config) -> Self {
        let mut state = match File::open(STATE) {
            Ok(mut file) => serde_json::from_reader(&mut file).unwrap(),
            Err(_) => {
                info!("can't find state, creating new state");
                SelfPlayState::default()
            }
        };
        loop {
            state = state.step(config);
            let mut file = File::create(STATE).unwrap();
            serde_json::to_writer(&mut file, &state).unwrap();
        }
    }
}
