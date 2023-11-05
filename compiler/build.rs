use lalrpop::Configuration;

fn main() {
    Configuration::new()
        .process_file("./src/passes/parse/grammar.lalrpop")
        .unwrap();
}
