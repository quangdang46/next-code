use next_code_best_of_n::BestOfNConfig;
use next_code_best_of_n::BestOfNMode;
use next_code_best_of_n::config::TemperatureStrategyConfig;
use next_code_best_of_n::strategies;

#[test]
fn test_config_effective_count_default() {
    let config = BestOfNConfig::default();
    assert!(config.enabled());
    assert_eq!(config.effective_count(), 4);
}

#[test]
fn test_config_off_mode_disabled() {
    let mut config = BestOfNConfig::default();
    config.mode = BestOfNMode::Off;
    assert!(!config.enabled());
}

#[test]
fn test_strategy_temperature_spread() {
    let temps = TemperatureStrategyConfig {
        min: 0.2,
        max: 0.8,
        values: vec![],
    };
    let strategies = strategies::generate_strategies(4, &temps);
    assert_eq!(strategies.len(), 4);
    let temp_values: Vec<f64> = strategies.iter().map(|s| s.temperature).collect();
    assert!(temp_values.windows(2).all(|w| w[0] < w[1]));
    assert!((temp_values[0] - 0.2).abs() < 0.01);
    assert!((temp_values[3] - 0.8).abs() < 0.01);
}
