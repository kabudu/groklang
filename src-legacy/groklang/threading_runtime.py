import threading
from queue import Queue
from typing import List, Any, Dict

class ThreadRuntime:
    def __init__(self):
        self.threads: Dict[int, threading.Thread] = {}
        self.thread_id = 0
        self.results: Dict[int, Any] = {}

    def spawn(self, func, args: List) -> int:
        """Spawn lightweight thread"""
        tid = self.thread_id
        self.thread_id += 1

        def wrapper():
            try:
                result = func(*args)
                self.results[tid] = result
            except Exception as e:
                self.results[tid] = f"Error: {e}"

        thread = threading.Thread(target=wrapper)
        self.threads[tid] = thread
        thread.start()
        return tid

    def join(self, tid: int) -> Any:
        """Wait for thread completion"""
        if tid in self.threads:
            self.threads[tid].join()
            return self.results.get(tid)
        raise ValueError(f"Thread {tid} not found")

class Mutex:
    def __init__(self):
        self.lock = threading.Lock()

    def acquire(self):
        self.lock.acquire()

    def release(self):
        self.lock.release()

class RwLock:
    def __init__(self):
        self.condition = threading.Condition()
        self.readers = 0
        self.writer = False

    def read_acquire(self):
        with self.condition:
            while self.writer:
                self.condition.wait()
            self.readers += 1

    def read_release(self):
        with self.condition:
            self.readers -= 1
            if self.readers == 0:
                self.condition.notify_all()

    def write_acquire(self):
        with self.condition:
            while self.readers > 0 or self.writer:
                self.condition.wait()
            self.writer = True

    def write_release(self):
        with self.condition:
            self.writer = False
            self.condition.notify_all()