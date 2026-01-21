#[cfg(test)]
mod tests {
    use grok::gc::{GarbageCollector, GcObject};

    #[test]
    fn test_gc_allocate_and_collect() {
        let mut gc = GarbageCollector::new();
        let id = gc.allocate(GcObject::Int(42));
        gc.add_root(id);
        gc.collect();
        // Should not collect root
        assert!(gc.objects.contains_key(&id));
    }
}
