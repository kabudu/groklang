use grok::ir::IRGenerator;
use grok::parser::Parser;
use grok::vm::VM;

#[tokio::test]
async fn test_deadlock_detection() {
    let parser = Parser::new();
    let input = "
        actor Waiter {
            let msg = receive {};
        }

        fn main() {
            let w = spawn Waiter {};
            // Main exits, but Waiter is still blocked.
            // Actually, if main exits, the VM might just return.
            // Let's make main also block.
            let msg = receive {};
        }
    ";
    let ast = parser.parse(input).unwrap();
    let mut gen = IRGenerator::new();
    let ir = gen.generate(&ast);

    let mut vm = VM::new();
    vm.load_program(&ir);

    let result = vm
        .execute(
            "main".to_string(),
            Some(tokio::sync::mpsc::unbounded_channel().1),
        )
        .await;

    assert!(result.is_err());
    assert!(result.err().unwrap().contains("deadlock"));
}

#[tokio::test]
async fn test_actor_failure_reporting() {
    let parser = Parser::new();
    let input = "
        actor Crashy {
            let x = 1 / 0; // Should cause error
        }

        fn main() {
            let c = spawn Crashy {};
            // Give it a moment to crash
        }
    ";
    let ast = parser.parse(input).unwrap();
    let mut gen = IRGenerator::new();
    let ir = gen.generate(&ast);

    let mut vm = VM::new();
    vm.load_program(&ir);

    let _ = vm.execute("main".to_string(), None).await;

    // In a real system, we'd check the registry status.
    // For now, check if it doesn't hang.
}
