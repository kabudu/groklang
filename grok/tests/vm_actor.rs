#[cfg(test)]
mod tests {
    use grok::parser::Parser;
    use grok::ir::IRGenerator;
    use grok::vm::VM;

    #[tokio::test]
    async fn test_actor_vm_execution() {
        let parser = Parser::new();
        let input = "
            actor Pong {
                let msg = receive {};
                // msg is 42
            }

            fn main() {
                let p = spawn Pong {};
                p ! 42;
            }
        ";
        let ast = parser.parse(input).unwrap();
        let mut gen = IRGenerator::new();
        let ir = gen.generate(&ast);
        
        let mut vm = VM::new();
        vm.load_program(&ir);
        let result = vm.execute("main".to_string(), None).await;
        assert!(result.is_ok(), "Execution failed: {:?}", result.err());
    }
}
