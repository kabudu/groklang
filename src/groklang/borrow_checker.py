from typing import Dict, Set, Tuple

class BorrowTracker:
    def __init__(self):
        self.borrows: Dict[int, Set[Tuple[str, int]]] = {}  # addr -> set of (borrow_type, borrow_id)

    def immutable_borrow(self, addr: int) -> int:
        """Create immutable borrow"""
        if addr not in self.borrows:
            self.borrows[addr] = set()
        borrow_id = id(object())  # Unique ID
        self.borrows[addr].add(('immutable', borrow_id))
        return borrow_id

    def mutable_borrow(self, addr: int) -> int:
        """Create mutable borrow (exclusive)"""
        if addr in self.borrows and len(self.borrows[addr]) > 0:
            raise RuntimeError(f"Cannot create mutable borrow while other borrows exist at {addr}")
        if addr not in self.borrows:
            self.borrows[addr] = set()
        borrow_id = id(object())
        self.borrows[addr].add(('mutable', borrow_id))
        return borrow_id

    def release_borrow(self, addr: int, borrow_id: int):
        """End borrow"""
        if addr in self.borrows:
            self.borrows[addr] = {b for b in self.borrows[addr] if b[1] != borrow_id}
            if not self.borrows[addr]:
                del self.borrows[addr]

    def check_access(self, addr: int, is_mutable: bool = False):
        """Check if access is allowed"""
        if addr not in self.borrows:
            return True  # No borrows, access OK

        borrows = self.borrows[addr]
        if is_mutable:
            # Mutable access requires no other borrows
            if len(borrows) > 0:
                raise RuntimeError(f"Mutable access not allowed while borrows exist at {addr}")
        else:
            # Immutable access OK if no mutable borrows
            if any(b[0] == 'mutable' for b in borrows):
                raise RuntimeError(f"Immutable access not allowed while mutable borrow exists at {addr}")
        return True