use import::*;

#[derive(Serialize, Deserialize)]
pub struct CycleResult {
    pub policy_data_path: PathBuf,
    pub value_data_path: PathBuf,
}

pub struct GameResult {
    pub outcome: Outcome,
    pub game_states: Vec<State>,
    pub value_instances: Vec<ValueInstance>,
    pub policy_instances: Vec<PolicyInstance>,
}

pub struct ValueInstance {
    pub whose_perspective: Player,
    pub feature_vec: FeatureVec,
}

impl ValueInstance {
    fn create(state: &State) -> Self {
        let feature_vec = features::featurize(state, state.available_moves().as_slice());
        let whose_perspective = state.current_player();
        ValueInstance { feature_vec, whose_perspective }
    }

    fn write(&mut self, w: impl Write, outcome: Outcome) {
        let label = match outcome {
            None => features::GameResult::Draw,
            Some(player) if player == self.whose_perspective =>
                features::GameResult::WhiteWin,
            Some(_) => features::GameResult::BlackWin,
        };
        self.feature_vec.write_libsvm(w, label as usize, |_| true);
    }
}

pub struct PolicyInstance {
    pub choice_made: usize,
    pub feature_vecs: Vec<FeatureVec>,
}

impl PolicyInstance {
    fn create(state: &State, mov: &Move) -> Option<Self> {
        let moves = state.available_moves();
        let moves = moves.as_slice();
        if moves.is_empty() {
            return None;
        }
        let choice_made = moves.iter().position(|x| x == mov)
            .unwrap_or_else(|| {
                for x in moves {
                    eprintln!("{}", x);
                }
                panic!("{} {}", moves.len(), mov)
            });
        let feature_vecs = moves.iter()
            .map(|mov| policy_features::featurize(state, mov))
            .collect();
        Some(PolicyInstance { choice_made, feature_vecs })
    }

    fn write(&mut self, mut w: impl Write) {
        writeln!(w, "{} 1:{}", self.feature_vecs.len(), self.choice_made).unwrap();
        for fv in &mut self.feature_vecs {
            fv.write_libsvm(&mut w, 0, |_| true);
        }
    }
}

fn new_game_state() -> State {
    let mut rng = rand::thread_rng();
    let mut state = State::default();
    for _ in 0..6 {
        let moves = state.available_moves();
        let moves = moves.as_slice();
        if moves.is_empty() {
            return state;
        }
        let choice_index = rng.gen_range(0, moves.len());
        state.make_move(&moves[choice_index]);
    }
    state
}

pub fn play(config: &Config, model: &Model) -> GameResult {
    let mut state = new_game_state();
    let mut value_instances = Vec::new();
    let mut policy_instances = Vec::new();
    let mut game_states = Vec::new();
    let outcome;
    loop {
        match make_single_move(config, model, &state) {
            Some(MoveResult {
                value_instances_from_sides,
                policy_instances_from_pv,
                new_state
            }) => {
                for value_instance in value_instances_from_sides {
                    value_instances.push(value_instance);
                }
                for policy_instance in policy_instances_from_pv {
                    policy_instances.push(policy_instance);
                }
                game_states.push(state);
                state = new_state;
            }
            None => {
                outcome = match state.outcome() {
                    BoardStatus::Ongoing => unreachable!(),
                    BoardStatus::Stalemate => None,
                    BoardStatus::Checkmate => {
                        if state.current_player() == Color::White {
                            Some(Color::Black)
                        } else {
                            Some(Color::White)
                        }
                    }
                };
                break;
            }
        }
    }
    debug!("finish game [{} moves]", game_states.len());
    GameResult { outcome, game_states, value_instances, policy_instances }
}

struct MoveResult {
    value_instances_from_sides: Vec<ValueInstance>,
    policy_instances_from_pv: Vec<PolicyInstance>,
    new_state: State,
}

fn make_single_move(config: &Config,
                    model: &Model,
                    state: &State)
                    -> Option<MoveResult> {
    let mut manager = Search::create_manager(state.clone(), model.clone());
    manager.playout_n(config.playouts_per_move);
    manager.best_move().map(|best_move| {
        let value_instances_from_sides = vec![
            ValueInstance::create(&state),
        ];
        let policy_instances_from_pv =
            manager.principal_variation(config.move_sample_depth)
            .into_iter()
            .scan(state.clone(), |state, mov| {
                let instance = PolicyInstance::create(state, &mov);
                state.make_move(&mov);
                instance
            })
            .collect();
        let mut new_state = state.clone();
        new_state.make_move(&best_move);
        MoveResult {
            value_instances_from_sides,
            policy_instances_from_pv,
            new_state,
        }
    })
}

pub fn play_cycle(config: &Config, model: &Model, iternum: &Iternum) -> CycleResult {
    let policy_data_path: PathBuf = iternum.name_for("policy_train");
    let value_data_path: PathBuf = iternum.name_for("value_train");
    let mut policy_file = BufWriter::new(File::create(policy_data_path.clone()).unwrap());
    let mut value_file = BufWriter::new(File::create(value_data_path.clone()).unwrap());
    let results: Vec<_> =
        (0..config.games_per_cycle)
        .into_par_iter()
        .map(|_| play(config, model))
        .collect();
    for game_result in results {
        let GameResult {
            outcome,
            game_states: _,
            mut value_instances,
            mut policy_instances,
        } = game_result;
        for v in &mut value_instances {
            v.write(&mut value_file, outcome);
        }
        for p in &mut policy_instances {
            p.write(&mut policy_file);
        }
    }
    CycleResult { policy_data_path, value_data_path }
}
