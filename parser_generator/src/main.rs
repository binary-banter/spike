use lalrpop::Configuration;

fn main() {
    Configuration::new().process_file("./compiler/src/passes/parse/grammar.lalrpop").unwrap();
}
