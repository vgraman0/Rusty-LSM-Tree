const NIL: usize = usize::MAX;

pub trait MemTable {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>);
    fn get(&self, key: &[u8]) -> Option<&[u8]>;
    fn delete(&mut self, key: &[u8]);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct SkipListMemTable {
    nodes: Vec<Node>,
    head: usize,
    max_level: usize,
    current_level: usize,
    len: usize,
}

struct Node {
    key: Vec<u8>,
    value: Option<Vec<u8>>,
    forward: Vec<usize>,
}

impl SkipListMemTable {
    pub fn new(max_level: usize) -> Self {
        let forward = vec![NIL; max_level];
        let head = Node {
            key: Vec::new(),
            value: None,
            forward,
        };
        let nodes = vec![head];

        SkipListMemTable {
            nodes,
            head: 0,
            max_level,
            current_level: 1,
            len: 0,
        }
    }

    fn random_level(&self) -> usize {
        let mut level = 1;
        while rand::random::<f64>() < 0.5 && level < self.max_level {
            level += 1;
        }
        level
    }

    fn find_update_path(&self, key: &[u8]) -> Vec<usize> {
        let mut updates = vec![0; self.current_level];
        let mut sentinel = self.head;
        for level in (0..self.current_level).rev() {
            let mut node = self.nodes[sentinel].forward[level];
            while node != NIL && self.nodes[node].key.as_slice() <= key {
                sentinel = node;
                node = self.nodes[sentinel].forward[level];
            }
            updates[level] = sentinel;
        }
        updates
    }
}

impl MemTable for SkipListMemTable {
    #[allow(clippy::needless_range_loop)]
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let updates = self.find_update_path(&key);
        if self.nodes[updates[0]].key == key {
            if self.nodes[updates[0]].value.is_none() {
                self.len += 1;
            }
            self.nodes[updates[0]].value = Some(value);
            return;
        }

        let node_level = self.random_level();
        let node_idx = self.nodes.len();
        let forward = vec![NIL; self.max_level];

        let node = Node {
            key,
            value: Some(value),
            forward,
        };
        self.nodes.push(node);

        for level in 0..node_level {
            if level < self.current_level {
                let prev = updates[level];
                let next = self.nodes[prev].forward[level];
                self.nodes[node_idx].forward[level] = next;
                self.nodes[prev].forward[level] = node_idx;
            } else {
                self.nodes[self.head].forward[level] = node_idx;
            }
        }

        self.len += 1;
        self.current_level = std::cmp::max(self.current_level, node_level);
    }

    fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let updates = self.find_update_path(key);
        if self.nodes[updates[0]].key == key {
            self.nodes[updates[0]].value.as_deref()
        } else {
            None
        }
    }

    fn delete(&mut self, key: &[u8]) {
        let updates = self.find_update_path(key);
        if self.nodes[updates[0]].key == key && self.nodes[updates[0]].value.is_some() {
            self.nodes[updates[0]].value = None;
            self.len -= 1;
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}
