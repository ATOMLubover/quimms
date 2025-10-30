use std::collections::HashMap;

pub type Hasher = fn(&str) -> u64;

/// Consistent Hash Ring implementation with virtual nodes (replicas).
pub struct ConsistentHashRing {
    hasher: Hasher,

    /// Stores the virtual node number on ring.
    ring: Vec<u64>,
    /// Maps from virtual node hash to real node identifier.
    virt_map: HashMap<u64, String>,
    /// Number of virtual nodes per real node.
    replicas: usize,
}

impl ConsistentHashRing {
    pub fn new(replicas: usize, hasher: Hasher) -> Self {
        Self {
            hasher,
            ring: Vec::new(),
            virt_map: HashMap::new(),
            replicas,
        }
    }

    pub fn add_node(&mut self, real_node: &str) {
        // Calculate hashes for every virtual nodes and add them to the ring.
        for i in 0..self.replicas {
            let virtual_node_id = format!("{}-{}", real_node, i);
            let hash = (self.hasher)(&virtual_node_id);

            self.ring.push(hash);

            self.virt_map.insert(hash, real_node.to_string());
        }

        // Keep the ring sorted after adding new nodes.
        self.ring.sort_unstable();
    }

    pub fn get_node(&self, key: &str) -> Option<&str> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = (self.hasher)(key);

        // Binary search to find the closest virtual node on the ring.
        match self.ring.binary_search(&hash) {
            Ok(index) => {
                // If exact match found, return the corresponding real node.
                self.virt_map.get(&self.ring[index]).map(|s| s.as_str())
            }
            Err(index) => {
                // If no exact match, wrap around the ring if necessary.
                let real_index = index % self.ring.len();

                self.virt_map
                    .get(&self.ring[real_index])
                    .map(|s| s.as_str())
            }
        }
    }

    pub fn remove_node(&mut self, real_node: &str) {
        // Remove all virtual nodes associated with the real node.
        for i in 0..self.replicas {
            let virtual_node_id = format!("{}-{}", real_node, i);
            let hash = (self.hasher)(&virtual_node_id);

            // Remove from ring
            if let Ok(pos) = self.ring.binary_search(&hash) {
                self.ring.remove(pos);
            }

            // Remove from virt_map
            self.virt_map.remove(&hash);
        }
    }

    pub fn clear(&mut self) {
        self.ring.clear();
        self.virt_map.clear();
    }

    pub fn len(&self) -> usize {
        self.virt_map.len()
    }
}
