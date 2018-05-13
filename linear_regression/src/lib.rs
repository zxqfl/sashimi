#[cfg(test)]
mod tests;

pub struct LinearRegression {
    coef: Vec<f64>,
    learning_rate: f64,
}

impl LinearRegression {
    pub fn new(num_predictors: usize, learning_rate: f64) -> Self {
        Self {
            coef: vec![0.0; num_predictors],
            learning_rate,
        }
    }

    pub fn update_coefficients(&mut self,
                               row: impl Iterator<Item=(usize, f64)> + Clone,
                               target: f64) {
        let prediction = self.evaluate(row.clone());
        let residual = target - prediction;
        for (index, value) in row {
            self.coef[index] += value * residual * self.learning_rate;
        }
    }

    pub fn evaluate(&self, row: impl Iterator<Item=(usize, f64)>) -> f64 {
        row.map(|(index, value)| self.coef[index] * value).sum()
    }

    pub fn coef(&self) -> &[f64] {
        &self.coef
    }
}
