use super::LinearRegression;
extern crate rand;
use self::rand::Rng;

#[test]
fn sanity() {
    let mut rng = rand::thread_rng();
    let num_predictors = 10;
    let true_coef: Vec<f64> = (0..num_predictors)
        .map(|_| rng.gen())
        .collect();
    let mut olr = LinearRegression::new(num_predictors);
    for _ in 0..1_000_000 {
        let values: Vec<f64> = (0..num_predictors)
            .map(|_| rng.gen())
            .collect();
        let target: f64 = values.iter().cloned().zip(true_coef.iter().cloned())
            .map(|(a, b)| a * b)
            .sum();
        olr.update_coefficients(values.iter().cloned().enumerate(), target);
    }
    println!("{:?}", true_coef);
    println!("{:?}", olr.coef());
    for i in 0..num_predictors {
        assert!(true_coef[i] - olr.coef()[i] < 1e-5);
    }
}
