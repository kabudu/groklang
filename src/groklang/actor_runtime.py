import threading
from queue import Queue
from typing import Any, Dict

class ChannelRuntime:
    def __init__(self):
        self.channels: Dict[int, Queue] = {}
        self.channel_id = 0

    def create_channel(self) -> int:
        """Create message channel"""
        cid = self.channel_id
        self.channel_id += 1
        self.channels[cid] = Queue()
        return cid

    def send(self, cid: int, message: Any):
        """Send message"""
        if cid in self.channels:
            self.channels[cid].put(message)
        else:
            raise ValueError(f"Channel {cid} not found")

    def recv(self, cid: int) -> Any:
        """Receive message (blocking)"""
        if cid in self.channels:
            return self.channels[cid].get()
        else:
            raise ValueError(f"Channel {cid} not found")

class Actor:
    def __init__(self, name: str):
        self.name = name
        self.mailbox = Queue()
        self.state: Dict[str, Any] = {}
        self.running = True

    def send_message(self, message: Any):
        """Send message to actor"""
        self.mailbox.put(message)

    def receive(self) -> Any:
        """Receive message (blocking)"""
        return self.mailbox.get()

    def run(self):
        """Actor event loop"""
        while self.running:
            msg = self.receive()
            if msg == 'EXIT':
                self.running = False
                break
            # Process message
            self.handle_message(msg)

    def handle_message(self, msg: Any):
        """Override in subclass"""
        pass

class ActorRuntime:
    def __init__(self):
        self.actors: Dict[str, Actor] = {}
        self.threads: Dict[str, threading.Thread] = {}

    def create_actor(self, actor_class, name: str) -> Actor:
        actor = actor_class(name)
        self.actors[name] = actor

        thread = threading.Thread(target=actor.run)
        self.threads[name] = thread
        thread.start()
        return actor

    def get_actor(self, name: str):
        return self.actors.get(name)

    def shutdown(self):
        for actor in self.actors.values():
            actor.send_message('EXIT')
        for thread in self.threads.values():
            thread.join()