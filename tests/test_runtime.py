from src.groklang.memory_manager import MemoryManager
from src.groklang.borrow_checker import BorrowTracker
from src.groklang.threading_runtime import ThreadRuntime, Mutex
from src.groklang.actor_runtime import ChannelRuntime, ActorRuntime, Actor

def test_memory_manager():
    mm = MemoryManager()
    addr = mm.allocate(42)
    assert mm.get_value(addr) == 42
    mm.clone_ref(addr)
    mm.drop_ref(addr)  # refcount still 1
    assert mm.get_value(addr) == 42
    mm.drop_ref(addr)  # now deallocated
    try:
        mm.get_value(addr)
        assert False, "Should have raised ValueError"
    except ValueError:
        pass
    print("Memory manager test passed!")

def test_borrow_checker():
    bc = BorrowTracker()
    addr = 1000
    borrow_id = bc.immutable_borrow(addr)
    assert bc.check_access(addr)  # OK
    bc.release_borrow(addr, borrow_id)

    # Mutable borrow
    borrow_id2 = bc.mutable_borrow(addr)
    assert bc.check_access(addr, is_mutable=True)  # OK
    bc.release_borrow(addr, borrow_id2)
    print("Borrow checker test passed!")

def test_threading():
    tr = ThreadRuntime()

    def add(a, b):
        return a + b

    tid = tr.spawn(add, [5, 10])
    result = tr.join(tid)
    assert result == 15
    print("Threading test passed!")

def test_channels():
    cr = ChannelRuntime()
    cid = cr.create_channel()
    cr.send(cid, "hello")
    msg = cr.recv(cid)
    assert msg == "hello"
    print("Channels test passed!")

def test_actors():
    ar = ActorRuntime()

    class TestActor(Actor):
        def handle_message(self, msg):
            if msg == "ping":
                self.state["received"] = True

    actor = ar.create_actor(TestActor, "test")
    actor.send_message("ping")
    import time
    time.sleep(0.1)  # Wait for processing
    ar.shutdown()
    print("Actors test passed!")

if __name__ == "__main__":
    test_memory_manager()
    test_borrow_checker()
    test_threading()
    test_channels()
    test_actors()