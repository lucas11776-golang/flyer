use std::collections::HashMap;

pub struct LruCache {
    nodes: Vec<Node>,
    head: Option<usize>,
    tail: Option<usize>,
    free: Vec<usize>,
    map: HashMap<String, usize>,
    total_size: usize,
    max_size: usize,
}

// TODO: expires in time.
#[derive(Clone)]
pub(crate) struct Asset {
    pub size: usize,
    pub expires: u128,
    pub data: Vec<u8>,
    pub content_type: String,
}

struct Node {
    key: String,
    value: Asset,
    prev: Option<usize>,
    next: Option<usize>,
    alive: bool,
}

impl LruCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            free: Vec::new(),
            map: HashMap::new(),
            total_size: 0,
            max_size,
        }
    }

    pub fn get(&mut self, key: &str, now: u128) -> Option<&Asset> {
        if let Some(&idx) = self.map.get(key) {
            if self.nodes[idx].value.expires != 0 && self.nodes[idx].value.expires <= now {
                self.remove_index(idx);

                return None;
            }

            self.move_to_head(idx);
            
            return Some(&self.nodes[idx].value);
        }
        None
    }

    pub fn insert(&mut self, key: String, value: Asset) {
        if let Some(&idx) = self.map.get(&key) {
            self.total_size = self.total_size + value.size - self.nodes[idx].value.size;
            self.nodes[idx].value = value;

            self.move_to_head(idx);
            self.evict_if_needed();
            
            return;
        }

        let idx = if let Some(free_idx) = self.free.pop() {
            self.nodes[free_idx] = Node {
                key: key.clone(),
                value: value.clone(),
                prev: None,
                next: None,
                alive: true,
            };
            
            free_idx
        } else {
            self.nodes.push(Node {
                key: key.clone(),
                value: value.clone(),
                prev: None,
                next: None,
                alive: true,
            });

            self.nodes.len() - 1
        };

        self.map.insert(key, idx);
        self.insert_at_head(idx);
        
        self.total_size += value.size;

        self.evict_if_needed();
    }

    fn insert_at_head(&mut self, idx: usize) {
        self.nodes[idx].prev = None;
        self.nodes[idx].next = self.head;

        if let Some(h) = self.head {
            self.nodes[h].prev = Some(idx);
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    fn move_to_head(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return;
        }

        let (prev, next) = (self.nodes[idx].prev, self.nodes[idx].next);

        if let Some(p) = prev {
            self.nodes[p].next = next;
        }

        if let Some(n) = next {
            self.nodes[n].prev = prev;
        }

        if self.tail == Some(idx) {
            self.tail = prev;
        }

        self.nodes[idx].prev = None;
        self.nodes[idx].next = self.head;

        if let Some(h) = self.head {
            self.nodes[h].prev = Some(idx);
        }

        self.head = Some(idx);
    }

    fn remove_index(&mut self, idx: usize) -> Asset {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;

        if let Some(p) = prev {
            self.nodes[p].next = next;
        }

        if let Some(n) = next {
            self.nodes[n].prev = prev;
        }

        if self.head == Some(idx) {
            self.head = next;
        }

        if self.tail == Some(idx) {
            self.tail = prev;
        }

        self.nodes[idx].alive = false;

        let node = std::mem::replace(&mut self.nodes[idx], Node {
            key: String::new(),
            value: Asset { size: 0, expires: 0, data: Vec::new(), content_type: String::new() },
            prev: None,
            next: None,
            alive: false,
        });

        self.map.remove(&node.key);
        self.free.push(idx);

        self.total_size = self.total_size.saturating_sub(node.value.size);

        return node.value
    }

    fn evict_if_needed(&mut self) {
        while self.total_size > self.max_size {
            if let Some(t) = self.tail {
                let _ = self.remove_index(t);
            } else {
                break;
            }
        }
    }
}