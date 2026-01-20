from typing import Dict, Set, List, Any
import gc
import weakref

class AdvancedGC:
    def __init__(self):
        self.objects: Dict[int, Any] = {}
        self.roots: Set[int] = set()
        self.ref_counts: Dict[int, int] = {}

    def allocate(self, obj: Any) -> int:
        """Allocate object with GC tracking"""
        obj_id = id(obj)
        self.objects[obj_id] = obj
        self.ref_counts[obj_id] = 1
        return obj_id

    def add_root(self, obj_id: int):
        """Mark object as GC root"""
        self.roots.add(obj_id)

    def mark_sweep(self):
        """Mark-sweep garbage collection"""
        marked = set()
        
        def mark(obj_id):
            if obj_id in marked or obj_id not in self.objects:
                return
            marked.add(obj_id)
            
            obj = self.objects[obj_id]
            # Mark referenced objects (simplified)
            if hasattr(obj, '__dict__'):
                for value in obj.__dict__.values():
                    if isinstance(value, int) and value in self.objects:
                        mark(value)
        
        # Mark from roots
        for root in self.roots:
            mark(root)
        
        # Sweep unmarked
        to_delete = []
        for obj_id in self.objects:
            if obj_id not in marked:
                to_delete.append(obj_id)
        
        for obj_id in to_delete:
            del self.objects[obj_id]
            del self.ref_counts[obj_id]
        
        return len(to_delete)

    def collect(self) -> int:
        """Run garbage collection"""
        return self.mark_sweep()

    def get_stats(self) -> Dict[str, int]:
        """Get GC statistics"""
        return {
            'objects': len(self.objects),
            'roots': len(self.roots),
            'freed': self.collect()
        }