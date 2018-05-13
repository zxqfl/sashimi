use import::*;

const LEARNING_RATE: f64 = 1e-3;

struct PastEvaluation {
    feature_vec: FeatureVec,
    value: i64,
    color: Color,
}

pub struct GooseEval {
    generic_model: Model,
    specific_model: Mutex<SpecificModel>,
    past_evaluations: Mutex<Vec<Box<PastEvaluation>>>,
}

pub struct NodeData(*const PastEvaluation);

unsafe impl Pod for NodeData {}
unsafe impl Send for NodeData {}
unsafe impl Sync for NodeData {}

pub struct SpecificModel([LinearRegression; 2]);

impl SpecificModel {
    pub fn new() -> Self {
        SpecificModel([
            LinearRegression::new(features::NUM_DENSE_FEATURES, LEARNING_RATE),
            LinearRegression::new(features::NUM_DENSE_FEATURES, LEARNING_RATE)])
    }

    fn model_for(&mut self, c: Color) -> &mut LinearRegression {
        &mut self.0[c as usize]
    }
}

#[derive(Clone, Copy)]
pub struct Evaluation {
    generic_evaluation_for_white: i64,
    specific_evaluation_for_white: i64,
}

impl Evaluation {
    pub fn draw() -> Self {
        Self {
            generic_evaluation_for_white: 0,
            specific_evaluation_for_white: 0,
        }
    }
}

fn quantize(x: f32) -> i64 {
    (x * SCALE) as i64
}
fn unquantize(x: i64) -> f64 {
    x as f64 * (1.0 / SCALE as f64)
}

impl Evaluator<GooseMCTS> for GooseEval {
    type StateEvaluation = Evaluation;

    fn evaluate_new_state(&self,
                          state: &State,
                          moves: &MoveList,
                          _handle: Option<SearchHandle<GooseMCTS>>)
                          -> (Vec<f32>, Evaluation, NodeData) {
        let move_evaluations = evaluate_moves(state, moves.as_slice(), &self.generic_model);
        let (state_evaluation, node_data) = if moves.len() == 0 {
            let scale = SCALE as i64;
            let generic = match state.outcome() {
                BoardStatus::Stalemate => 0,
                BoardStatus::Checkmate => {
                    if state.board().side_to_move() == Color::White {
                        -scale
                    } else {
                        scale
                    }
                }
                BoardStatus::Ongoing => unreachable!(),
            };
            (Evaluation {
                generic_evaluation_for_white: generic,
                specific_evaluation_for_white: 0,
            },
             null())
        } else {
            let (generic, specific) = features::score(
                state, moves.as_slice(),
                &self.generic_model,
                &self.specific_model.lock()
                    .expect("lock model")
                    .model_for(state.board().side_to_move()));
            let past_evaluation = PastEvaluation {
                feature_vec: features::featurize(state,
                                                 moves.as_slice(),
                                                 WhichFeatures::OnlyDenseFeatures),
                value: quantize(generic),
                color: state.board().side_to_move(),
            };
            let past_evaluation = Box::new(past_evaluation);
            let node_data = &*past_evaluation as *const PastEvaluation;
            self.past_evaluations.lock()
                .expect("lock past_evaluations")
                .push(past_evaluation);
            (Evaluation {
                generic_evaluation_for_white: quantize(generic),
                specific_evaluation_for_white: quantize(specific),
            },
             node_data)
        };
        (move_evaluations, state_evaluation, NodeData(node_data))
    }

    fn evaluate_existing_state(&self, _: &State, evaln: &Evaluation,
                               _: SearchHandle<GooseMCTS>) -> Evaluation {
        *evaln
    }

    fn interpret_evaluation_for_player(&self, evaln: &Evaluation, player: &Player) -> i64 {
        let evaln = evaln.generic_evaluation_for_white + evaln.specific_evaluation_for_white;
        match *player {
            Color::White => evaln,
            Color::Black => -evaln,
        }
    }

    fn on_backpropagation(&self,
                          evaln: &Evaluation,
                          handle: SearchHandle<GooseMCTS>) {
        if rand::thread_rng().gen_range(0, 16) != 0 {
            return;
        }
        let data = handle.node().data().0;
        if data != null() {
            let past_evaluation = unsafe { &*data };
            let mut specific_model = self.specific_model.lock().unwrap();
            let residual = evaln.generic_evaluation_for_white - past_evaluation.value;
            let residual = match past_evaluation.color {
                Color::White => residual,
                Color::Black => -residual,
            };
            specific_model.model_for(past_evaluation.color)
                .update_coefficients(past_evaluation.feature_vec.iter(),
                                     unquantize(residual));
        }
    }
}

impl GooseEval {
    pub fn new(generic_model: Model, specific_model: SpecificModel) -> Self {
        let specific_model = Mutex::new(specific_model);
        Self {
            generic_model,
            specific_model,
            past_evaluations: Vec::new().into(),
        }
    }

    pub fn into_specific_model(self) -> SpecificModel {
        self.specific_model.into_inner().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use search::Search;
    use mcts::GameState;
    use super::*;
    use float_ord::FloatOrd;

    fn assert_find_move(fen: &str, desired: &str) -> Vec<State> {
        let pv_len = 15;
        let state = State::from_fen(fen).unwrap();
        let moves = state.available_moves();
        let moves = moves.as_slice();
        let evalns = evaluate_moves(&state, &moves, &Model::default());
        let mut paired: Vec<_> = moves.iter().zip(evalns.iter()).collect();
        paired.sort_by_key(|x| FloatOrd(*x.1));
        for (a, b) in paired {
            println!("policy: {} {}", a, b);
        }
        let mut manager = Search::create_manager(state,
                                                 Model::default(),
                                                 SpecificModel::new());
        // for _ in 0..5 {
        manager.playout_n(1_000_000);
        println!("\n\nMOVES");
        manager.tree().display_moves();
        // }
        println!("Principal variation");
        let mov = manager.best_move().unwrap();
        for state in manager.principal_variation_states(pv_len) {
            println!("{}", state.board());
        }
        for info in manager.principal_variation_info(pv_len) {
            println!("{}", info);
        }
        println!("{}", manager.tree().diagnose());
        assert!(format!("{}", mov).starts_with(desired),
                "expected {}, got {}",
                desired,
                mov);
        manager.principal_variation_states(pv_len)
    }

    #[test]
    fn mate_in_one() {
        assert_find_move("6k1/8/6K1/8/8/8/8/R7 w - - 0 0", "a1-a8");;
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn mate_in_six() {
        assert_find_move("5q2/6Pk/8/6K1/8/8/8/8 w - - 0 0", "g7-f8=R");
    }

    #[test]
    #[ignore]
    fn take_the_bishop() {
        assert_find_move("r3k2r/ppp1q1pp/2n1b3/8/3p4/6p1/PPPNQPP1/2K1RB1R w kq - 0 16", "Re1xe6");
    }

    #[test]
    #[ignore]
    fn what_happened() {
        assert_find_move("2k1r3/ppp2pp1/2nb1n1p/1q1rp3/8/2QPBNPP/PP2PPBK/2RR4 b - - 9 20", "foo");
    }

    #[test]
    #[ignore]
    fn what_happened_2() {
        assert_find_move("2r4r/ppB3p1/2n2k1p/1N5q/1b3Qn1/6Pb/PP2PPBP/R4RK1 b - - 10 18", "foo");
    }

    #[test]
    #[ignore]
    fn checkmating() {
        let states = assert_find_move("8/8/8/3k4/1Q6/K7/8/8 w - - 8 59", "");
        assert!(states[states.len() - 1].outcome() == BoardStatus::Checkmate);
    }

    #[test]
    #[ignore]
    fn interesting() {
        assert_find_move("2kr4/pp2bp1p/3p4/5b1Q/4q1r1/N4P2/PPPP2PP/R1B2RK1 b - -", "?");
    }
}
