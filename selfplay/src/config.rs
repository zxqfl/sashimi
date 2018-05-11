#[derive(Deserialize)]
pub struct Config {
    pub playouts_per_move: u64,
    pub games_per_cycle: usize,
    pub move_sample_depth: usize,
    pub training_window_length: usize,
}
