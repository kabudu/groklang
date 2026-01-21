#[cfg(test)]
mod tests {
    use grok::ast::AstNode;
    use grok::ir::{IRFunction, IRGenerator, IRInstruction};
    use grok::vm::VM;

    #[test]
    fn test_ir_generation() {
        let gen = IRGenerator::new();
        let ast = AstNode::Program(vec![]);
        let ir = gen.generate(&ast);
        assert_eq!(ir.len(), 0); // Empty program
    }

    #[test]
    fn test_vm_execution() {
        let mut vm = VM::new();
        let func = IRFunction {
            name: "test".to_string(),
            instructions: vec![IRInstruction::Return],
        };
        vm.load_program(&[func]);
        assert!(vm.execute("test").is_ok());
    }
}
