//! Garbage Collector for Vāktra VM
//! 
//! Advanced garbage collection inspired by Vedic concepts
//! of renewal (नवीकरण) and purification (शुद्धीकरण).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::{VmError, VmResult};
use crate::stack::VmStack;
use vaaktra_jit::runtime::RuntimeValue;

/// Garbage collector for the VM
pub struct GarbageCollector {
    /// Heap objects
    heap: HashMap<ObjectId, HeapObject>,
    
    /// Next object ID
    next_id: ObjectId,
    
    /// GC statistics
    stats: GcStats,
    
    /// GC configuration
    config: GcConfig,
}

/// Object ID type
type ObjectId = u64;

/// Heap object
#[derive(Debug, Clone)]
pub struct HeapObject {
    pub id: ObjectId,
    pub value: RuntimeValue,
    pub references: Vec<ObjectId>,
    pub mark: bool,
    pub generation: Generation,
    pub size: usize,
}

/// Object generation for generational GC
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Generation {
    Young,
    Old,
}

/// GC statistics
#[derive(Debug, Default)]
pub struct GcStats {
    pub collections: u64,
    pub objects_collected: u64,
    pub bytes_collected: u64,
    pub total_allocations: u64,
    pub total_bytes_allocated: u64,
}

/// GC configuration
#[derive(Debug)]
pub struct GcConfig {
    /// Enable generational collection
    pub generational: bool,
    
    /// Young generation threshold
    pub young_threshold: usize,
    
    /// Collection frequency
    pub collection_frequency: u32,
    
    /// Enable incremental collection
    pub incremental: bool,
}

impl GarbageCollector {
    /// Create a new garbage collector
    pub fn new() -> VmResult<Self> {
        Ok(GarbageCollector {
            heap: HashMap::new(),
            next_id: 1,
            stats: GcStats::default(),
            config: GcConfig {
                generational: true,
                young_threshold: 1000,
                incremental: true,
                collection_frequency: 100,
            },
        })
    }
    
    /// Allocate an object on the heap
    pub fn allocate(&mut self, value: RuntimeValue) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        
        let size = self.estimate_size(&value);
        let object = HeapObject {
            id,
            value,
            references: Vec::new(),
            mark: false,
            generation: Generation::Young,
            size,
        };
        
        self.heap.insert(id, object);
        self.stats.total_allocations += 1;
        self.stats.total_bytes_allocated += size as u64;
        
        id
    }
    
    /// Perform garbage collection
    pub fn collect(
        &mut self, 
        stack: &VmStack, 
        globals: &Arc<RwLock<HashMap<String, RuntimeValue>>>
    ) -> VmResult<usize> {
        log::debug!("Starting garbage collection");
        
        let initial_count = self.heap.len();
        let initial_size = self.heap_size();
        
        // Mark phase
        self.mark_phase(stack, globals)?;
        
        // Sweep phase
        let collected = self.sweep_phase()?;
        
        // Update statistics
        self.stats.collections += 1;
        self.stats.objects_collected += collected as u64;
        self.stats.bytes_collected += (initial_size - self.heap_size()) as u64;
        
        log::debug!("GC collected {} objects, {} bytes", collected, initial_size - self.heap_size());
        
        Ok(collected)
    }
    
    /// Mark phase - mark all reachable objects
    fn mark_phase(
        &mut self, 
        stack: &VmStack, 
        globals: &Arc<RwLock<HashMap<String, RuntimeValue>>>
    ) -> VmResult<()> {
        // Clear all marks
        for object in self.heap.values_mut() {
            object.mark = false;
        }
        
        // Mark objects reachable from stack
        for value in stack.values() {
            self.mark_value(value);
        }
        
        // Mark objects reachable from globals
        let globals_read = globals.read();
        for value in globals_read.values() {
            self.mark_value(value);
        }
        drop(globals_read);
        
        // Mark transitively reachable objects
        self.mark_transitive();
        
        Ok(())
    }
    
    /// Mark a value and its references
    fn mark_value(&mut self, value: &RuntimeValue) {
        match value {
            RuntimeValue::Suchi(list) => {
                for item in list {
                    self.mark_value(item);
                }
            }
            RuntimeValue::Nidhaan(map) => {
                for item_value in map.values() {
                    self.mark_value(item_value);
                }
            }
            RuntimeValue::Dharma(obj) => {
                for field_value in obj.values() {
                    self.mark_value(field_value);
                }
            }
            _ => {
                // For other types, no additional marking needed
            }
        }
    }
    
    /// Mark transitively reachable objects
    fn mark_transitive(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            
            let marked_ids: Vec<ObjectId> = self.heap.iter()
                .filter(|(_, obj)| obj.mark)
                .map(|(id, _)| *id)
                .collect();
            
            for id in marked_ids {
                if let Some(object) = self.heap.get(&id) {
                    let references = object.references.clone();
                    for ref_id in references {
                        if let Some(ref_obj) = self.heap.get_mut(&ref_id) {
                            if !ref_obj.mark {
                                ref_obj.mark = true;
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Sweep phase - collect unmarked objects
    fn sweep_phase(&mut self) -> VmResult<usize> {
        let mut collected = 0;
        let mut to_remove = Vec::new();
        
        for (id, object) in &self.heap {
            if !object.mark {
                to_remove.push(*id);
                collected += 1;
            }
        }
        
        // Remove unmarked objects
        for id in to_remove {
            self.heap.remove(&id);
        }
        
        // Promote surviving young objects to old generation
        if self.config.generational {
            for object in self.heap.values_mut() {
                if object.generation == Generation::Young && object.mark {
                    object.generation = Generation::Old;
                }
            }
        }
        
        Ok(collected)
    }
    
    /// Estimate the size of a runtime value
    fn estimate_size(&self, value: &RuntimeValue) -> usize {
        match value {
            RuntimeValue::Sankhya(_) => 8,
            RuntimeValue::Satyasatya(_) => 1,
            RuntimeValue::Shabda(s) => s.len() + 24, // String overhead
            RuntimeValue::Suchi(list) => {
                let mut size = 24; // Vec overhead
                for item in list {
                    size += self.estimate_size(item);
                }
                size
            }
            RuntimeValue::Nidhaan(map) => {
                let mut size = 48; // HashMap overhead
                for (key, value) in map {
                    size += key.len() + self.estimate_size(value);
                }
                size
            }
            RuntimeValue::Dharma(obj) => {
                let mut size = 48; // HashMap overhead
                for (key, value) in obj {
                    size += key.len() + self.estimate_size(value);
                }
                size
            }
            RuntimeValue::Shunya => 0,
            RuntimeValue::Mantra(_) => 8, // Function pointer
        }
    }
    
    /// Get total heap size
    pub fn heap_size(&self) -> usize {
        self.heap.values().map(|obj| obj.size).sum()
    }
    
    /// Get number of objects in heap
    pub fn object_count(&self) -> usize {
        self.heap.len()
    }
    
    /// Get GC statistics
    pub fn get_stats(&self) -> &GcStats {
        &self.stats
    }
    
    /// Configure the garbage collector
    pub fn configure(&mut self, config: GcConfig) {
        self.config = config;
    }
    
    /// Check if GC should run
    pub fn should_collect(&self) -> bool {
        if self.config.generational {
            let young_count = self.heap.values()
                .filter(|obj| obj.generation == Generation::Young)
                .count();
            young_count >= self.config.young_threshold
        } else {
            self.heap.len() >= self.config.young_threshold
        }
    }
    
    /// Perform incremental collection (collect only young generation)
    pub fn collect_young(
        &mut self, 
        stack: &VmStack, 
        globals: &Arc<RwLock<HashMap<String, RuntimeValue>>>
    ) -> VmResult<usize> {
        if !self.config.generational {
            return self.collect(stack, globals);
        }
        
        log::debug!("Starting young generation collection");
        
        let initial_young_count = self.heap.values()
            .filter(|obj| obj.generation == Generation::Young)
            .count();
        
        // Mark phase (same as full collection)
        self.mark_phase(stack, globals)?;
        
        // Sweep only young generation
        let mut collected = 0;
        let mut to_remove = Vec::new();
        
        for (id, object) in &self.heap {
            if object.generation == Generation::Young && !object.mark {
                to_remove.push(*id);
                collected += 1;
            }
        }
        
        // Remove unmarked young objects
        for id in to_remove {
            self.heap.remove(&id);
        }
        
        // Promote surviving young objects
        for object in self.heap.values_mut() {
            if object.generation == Generation::Young && object.mark {
                object.generation = Generation::Old;
            }
        }
        
        self.stats.collections += 1;
        self.stats.objects_collected += collected as u64;
        
        log::debug!("Young GC collected {} objects", collected);
        
        Ok(collected)
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create default GarbageCollector")
    }
}
