from src.groklang.deadlock_detector import DeadlockDetector
from src.groklang.decorator_processor import MockLlmService
from src.groklang.actor_runtime import Actor, ActorRuntime

def test_deadlock_detection():
    llm_service = MockLlmService()
    detector = DeadlockDetector(llm_service)

    code = """
    fn concurrent_task() {
        let ch1 = channel::create();
        let ch2 = channel::create();
        
        spawn {
            ch1.send(1);
            let val = ch2.recv();
        };
        
        spawn {
            ch2.send(2);
            let val = ch1.recv();
        };
    }
    """

    analysis = detector.analyze_code(code)
    assert 'risk_level' in analysis
    assert 'recommendations' in analysis
    print("Deadlock detection test passed!")

class TestActorImpl(Actor):
    def handle_message(self, msg):
        if msg == "test":
            self.state["received"] = True

def test_actor_supervision():
    runtime = ActorRuntime()
    
    # Create supervisor
    supervisor = runtime.create_actor(TestActorImpl, "supervisor")
    
    # Create child
    child = runtime.create_actor(TestActorImpl, "child", supervisor)
    
    # Test supervision
    assert child.supervisor == supervisor
    assert child in supervisor.children
    
    # Simulate failure
    child.handle_failure(ValueError("test error"))
    
    print("Actor supervision test passed!")

if __name__ == "__main__":
    test_deadlock_detection()
    test_actor_supervision()