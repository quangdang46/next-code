//! Ad-hoc e2e check: fetch models.dev live and resolve a few model prices.
//! Run: cargo run --example pricing_e2e_check
fn main() {
    next_code::model_pricing::schedule_refresh();
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some(c) = next_code::model_pricing::lookup("claude:api-key", "claude-fable-5") {
            println!("fable-5: {:?}", c);
            println!(
                "deepseek-v4-flash: {:?}",
                next_code::model_pricing::lookup("openai-compatible:deepseek", "deepseek-v4-flash")
            );
            println!(
                "kimi-k2.5: {:?}",
                next_code::model_pricing::lookup("openai-compatible:moonshotai", "kimi-k2.5")
            );
            println!(
                "openrouter kimi: {:?}",
                next_code::model_pricing::lookup("openrouter", "moonshotai/kimi-k2.5")
            );
            return;
        }
    }
    eprintln!("TIMEOUT: no pricing after 20s");
    std::process::exit(1);
}
