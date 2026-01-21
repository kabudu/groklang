import threading
from queue import Queue
from typing import Any, Dict, List

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
    def __init__(self, name: str, supervisor=None):
        self.name = name
        self.mailbox = Queue()
        self.state: Dict[str, Any] = {}
        self.running = True
        self.supervisor = supervisor
        self.children: List['Actor'] = []

    def send_message(self, message: Any):
        """Send message to actor"""
        self.mailbox.put(message)

    def receive(self) -> Any:
        """Receive message (blocking)"""
        return self.mailbox.get()

    def run(self):
        """Actor event loop"""
        try:
            while self.running:
                msg = self.receive()
                if msg == 'EXIT':
                    self.running = False
                    break
                elif isinstance(msg, tuple) and msg[0] == 'CHILD_FAILED':
                    self.handle_child_failure(msg[1], msg[2])
                else:
                    # Process message
                    self.handle_message(msg)
        except Exception as e:
            self.handle_failure(e)

    def handle_message(self, msg: Any):
        """Override in subclass"""
        pass

    def handle_failure(self, error: Exception):
        """Handle failure, notify supervisor"""
        if self.supervisor:
            self.supervisor.send_message(('CHILD_FAILED', self.name, str(error)))
        self.running = False

    def handle_child_failure(self, child_name: str, error: str):
        """Supervise child failure"""
        print(f"Supervisor {self.name}: Child {child_name} failed with {error}")
        # Restart logic (simplified)
        for child in self.children:
            if child.name == child_name and not child.running:
                # Restart child
                new_child = type(child)(child_name + '_restarted')
                self.children.remove(child)
                self.add_child(new_child)
                thread = threading.Thread(target=new_child.run)
                thread.start()
                break

    def add_child(self, child: 'Actor'):
        """Add supervised child"""
        self.children.append(child)
        child.supervisor = self

class ActorRuntime:
    def __init__(self):
        self.actors: Dict[str, Actor] = {}
        self.threads: Dict[str, threading.Thread] = {}

    def create_actor(self, actor_class, name: str, supervisor=None) -> Actor:
        actor = actor_class(name, supervisor)
        self.actors[name] = actor

        if supervisor:
            supervisor.add_child(actor)

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