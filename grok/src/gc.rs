use std::collections::HashMap;

pub struct GarbageCollector {
    pub objects: HashMap<usize, GcObject>,
    roots: Vec<usize>,
}

#[derive(Clone)]
pub enum GcObject {
    Int(i64),
    String(String),
    // Add more as needed
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            roots: Vec::new(),
        }
    }

    pub fn allocate(&mut self, obj: GcObject) -> usize {
        let id = self.objects.len();
        self.objects.insert(id, obj);
        id
    }

    pub fn collect(&mut self) {
        // Mark-sweep GC
        let mut marked = std::collections::HashSet::new();
        for &root in &self.roots {
            self.mark(root, &mut marked);
        }
        self.sweep(&marked);
    }

    fn mark(&self, id: usize, marked: &mut std::collections::HashSet<usize>) {
        if marked.contains(&id) {
            return;
        }
        marked.insert(id);
        // Mark references (placeholder for complex objects)
    }

    fn sweep(&mut self, marked: &std::collections::HashSet<usize>) {
        self.objects.retain(|id, _| marked.contains(id));
    }

    pub fn add_root(&mut self, id: usize) {
        self.roots.push(id);
    }
}
