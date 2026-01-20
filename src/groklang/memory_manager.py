from typing import Dict, Any

class MemoryManager:
    def __init__(self):
        self.allocations: Dict[int, list] = {}  # address -> [value, refcount]
        self.next_addr = 1000

    def allocate(self, value: Any) -> int:
        """Allocate on heap"""
        addr = self.next_addr
        self.allocations[addr] = [value, 1]  # [value, refcount]
        self.next_addr += 1
        return addr

    def clone_ref(self, addr: int):
        """Increment refcount"""
        if addr in self.allocations:
            self.allocations[addr][1] += 1

    def drop_ref(self, addr: int):
        """Decrement refcount, deallocate if zero"""
        if addr in self.allocations:
            self.allocations[addr][1] -= 1
            if self.allocations[addr][1] == 0:
                del self.allocations[addr]

    def get_value(self, addr: int) -> Any:
        """Get value at address"""
        if addr in self.allocations:
            return self.allocations[addr][0]
        raise ValueError(f"Invalid address: {addr}")

    def set_value(self, addr: int, value: Any):
        """Set value at address"""
        if addr in self.allocations:
            self.allocations[addr][0] = value
        else:
            raise ValueError(f"Invalid address: {addr}")