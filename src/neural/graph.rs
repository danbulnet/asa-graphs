use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap
};

use bionet_common::{
    sensor::{ Sensor, SensorData },
    neuron::{ Neuron, NeuronID },
    data::DataCategory
};

use super::{
    element::Element,
    node::Node
};

pub struct ASAGraph<Key, const ORDER: usize = 25>
where Key: SensorData + 'static, [(); ORDER + 1]: {
    pub name: Rc<str>,
    pub data_category: DataCategory,
    pub(crate) root: Rc<RefCell<Node<Key, ORDER>>>,
    pub element_min: Option<Rc<RefCell<Element<Key, ORDER>>>>,
    pub element_max: Option<Rc<RefCell<Element<Key, ORDER>>>>,
    pub key_min: Option<Key>,
    pub key_max: Option<Key>
}

impl<Key, const ORDER: usize> Sensor for ASAGraph<Key, ORDER> 
where Key: SensorData, [(); ORDER + 1]: {
    type Data = Key;

    fn name(&self) -> &str { &*self.name }

    fn data_category(&self) -> DataCategory { self.data_category }

    fn insert(&mut self, key: &Key) -> Rc<RefCell<dyn Neuron>> {
        self.insert(key)
    }

    fn search(&self, key: &Key) -> Option<Rc<RefCell<dyn Neuron>>> { 
        Some(self.search(key).unwrap() as Rc<RefCell<dyn Neuron>>) 
    }

    fn activate(
        &mut self, item: &Key, signal: f32, propagate_horizontal: bool, propagate_vertical: bool
    ) -> Result<HashMap<NeuronID, Rc<RefCell<dyn Neuron>>>, String> {
        let element = match self.search(item) {
            Some(e) => e,
            None => { 
                match self.data_category {
                    DataCategory::Categorical => {
                        log::error!("activating missing categorical sensory neuron {}", item);
                        return Err(
                            format!("activating missing categorical sensory neuron {}", item)
                        )
                    },
                    DataCategory::Numerical | DataCategory::Ordinal => {
                        if propagate_horizontal {
                            log::warn!(
                                "activating missing non-categorical sensory neuron {}, inserting",
                                item
                            );
                            self.insert(&item)
                        } else {
                            log::error!(
                                "activating missing non-categorical sensory neuron {} with {}",
                                item, "propagate_horizontal=false"
                            );
                            return Err(format!(
                                "activating missing non-categorical sensory neuron {} with {}",
                                item, "propagate_horizontal=false"
                            ))
                        }
                    }
                }
            }
        };

        Ok(element.clone().borrow_mut().activate(signal, propagate_horizontal, propagate_vertical))
    }

    fn deactivate(
        &mut self, item: &Key, propagate_horizontal: bool, propagate_vertical: bool
    ) -> Result<(), String> {
        let element = match self.search(item) {
            Some(e) => e,
            None => {
                log::error!("deactivating non-existing sensory neuron {}", item);
                return Err(format!("deactivating non-existing sensory neuron {}", item))
            }
        };

        element.borrow_mut().deactivate(propagate_horizontal, propagate_vertical);

        Ok(())
    }

    fn deactivate_sensor(&mut self) {
        let mut element = match &self.element_min {
            Some(e) => e.clone(),
            None => { log::warn!("no element_min in asa-graph"); return }
        };

        loop {
            element.borrow_mut().deactivate(false, false);

            let new_element= match &element.borrow().next {
                Some(e) => e.0.upgrade().unwrap().clone(),
                None => break
            };
            element = new_element;
        }
    }
}

impl<Key, const ORDER: usize> ASAGraph<Key, ORDER> 
where Key: SensorData, [(); ORDER + 1]: {
    pub fn new(name: &str, data_category: DataCategory) -> ASAGraph<Key, ORDER> {
        if ORDER < 3 {
            panic!("Graph order must be >= 3");
        }
        ASAGraph {
            name: Rc::from(name),
            data_category,
            root: Rc::new(RefCell::new(Node::<Key, ORDER>::new(true, None))),
            element_min: None,
            element_max: None,
            key_min: None,
            key_max: None
        }
    }

    pub fn new_rc(name: &str, data_category: DataCategory) -> Rc<RefCell<ASAGraph<Key, ORDER>>> {
        if ORDER < 3 {
            panic!("Graph order must be >= 3");
        }
        Rc::new(RefCell::new(ASAGraph::new(name, data_category)))
    }

    pub fn search(&self, key: &Key) -> Option<Rc<RefCell<Element<Key, ORDER>>>> {
        let node = &self.root;
        
        let (key_min, key_max) = self.extreme_keys()?;

        if key.distance(key_max) > key.distance(key_min) {
            return Self::search_left(key, &*node.borrow())
        } else {
            return Self::search_right(key, &*node.borrow())
        }
    }

    fn search_left<'a, 'b>(
        key: &'a Key, mut node: &'b Node<Key, ORDER>
    ) -> Option<Rc<RefCell<Element<Key, ORDER>>>> {
        loop {
            let mut index = 0;
            {
                let mut current_key = node.keys[index].as_ref().unwrap();
                
                while index < node.size && key > current_key {
                    index += 1;
                    if index < node.size {
                        current_key = node.keys[index].as_ref().unwrap();
                    }
                }

                if index < node.size && key == current_key {
                    let element = node.elements[index].as_ref().unwrap().clone();
                    return Some(element)
                } else if node.is_leaf {
                    return None
                }
            }
                
            let node_ptr = node.children[index].as_ref().unwrap();
            unsafe { node = node_ptr.try_borrow_unguarded().unwrap() };
        }
    }

    fn search_right<'a, 'b>(
        key: &'a Key, mut node: &'b Node<Key, ORDER>
    ) -> Option<Rc<RefCell<Element<Key, ORDER>>>> {
        loop {
            let mut index = node.size - 1;
            {
                let mut current_key = node.keys[index].as_ref().unwrap();
                
                while index > 0 && key < current_key {
                    index -= 1;
                    current_key = node.keys[index].as_ref().unwrap();
                }

                if key == current_key {
                    let element = node.elements[index].as_ref().unwrap().clone();
                    return Some(element)
                } else if node.is_leaf {
                    return None
                } else if key > current_key {
                    index += 1;
                }
            }
            let node_ptr = node.children[index].as_ref().unwrap();
            unsafe { node = node_ptr.try_borrow_unguarded().unwrap() };
        }
    }

    pub fn insert(&mut self, key: &Key) -> Rc<RefCell<Element<Key, ORDER>>> {
        let mut node = self.root.clone();

        if node.borrow().size == 0 { return self.insert_first_element(&node, key) }

        if node.borrow().size == Node::<Key, ORDER>::MAX_KEYS { node = self.split_root(); }

        let (key_min, key_max) = self.extreme_keys().unwrap_or_else(|| {
            panic!("element_min / element_min must not be nullptr")
        });

        loop {
            let node_insert_result = if key.distance(key_max) > key.distance(key_min) {
                node.borrow().insert_existing_key(key, true)
            } else {
                node.borrow().insert_existing_key(key, false)
            };
            if let Some(el) = node_insert_result.0 { return el }
            let mut index = node_insert_result.1;
    
            if node.borrow().is_leaf {
                let element = Node::insert_key_leaf(&node, key, &self.name, self.range());
                self.set_extrema(&element);
                return element
            } else {
                let child_size = node.borrow().children[index].as_ref().unwrap().borrow().size;
                if child_size == Node::<Key, ORDER>::MAX_KEYS {
                    Node::split_child(&node, index);
                    if key > &node.borrow().elements[index].as_ref().unwrap().borrow().key {
                        index += 1 
                    } else if key == node.borrow().keys[index].as_ref().unwrap() {
                        return node.borrow().elements[index].as_ref().unwrap().clone()
                    }
                }
                let new_node = node.borrow().children[index].as_ref().unwrap().clone();
                node = new_node.clone();
            }
        }
    }

    pub fn range(&self) -> f32 { 
        if self.key_min.is_none() || self.key_max.is_none() { return f32::NAN }
        let ret = self.key_min.as_ref().unwrap().distance(self.key_max.as_ref().unwrap()) as f32;
        if ret == 0.0f32 { 1.0f32 } else { ret }
     }

    pub fn print_graph(&self) {
        let mut height = 0;
        let mut node = self.root.clone();
        let mut queue: Vec<Vec<Rc<RefCell<Node<Key, ORDER>>>>> = vec![vec![]];
        queue[0].push(node.clone());

        loop {
            queue.push(vec![]);
            for i in 0..(queue[height].len()) {
                node = queue[height][i].clone();
                let node_size = node.borrow().size;
                print!("||");
                for j in 0..(node_size) {
                    let element = node.borrow().elements[j].as_ref().unwrap().clone();
                    print!("{}:{}|", &element.borrow().key, element.borrow().counter);
                    if !node.borrow().is_leaf {
                        queue[height + 1].push(node.borrow().children[j].as_ref().unwrap().clone());
                    }
                }
                if !node.borrow().is_leaf {
                    queue[height + 1].push(node.borrow().children[node_size].as_ref().unwrap().clone());
                }
                print!("| ");
            }
            if queue.last().unwrap().len() > 0 {
                height += 1;
                println!("");
            } else {
                println!("");
                return
            }
        }
    }

    fn extreme_keys<'a>(&'a self) -> Option<(&'a Key, &'a Key)> {
        if self.key_min.is_none() || self.key_max.is_none() { return None }
        let key_min =  self.key_min.as_ref().unwrap();
        let key_max =  self.key_max.as_ref().unwrap();
        Some((key_min, key_max))
    }

    fn insert_first_element(
        &mut self, node: &Rc<RefCell<Node<Key, ORDER>>>,  key: &Key
    ) -> Rc<RefCell<Element<Key, ORDER>>> {
        let element_pointer = Element::<Key, ORDER>::new(key, &self.name);
        node.borrow_mut().elements[0] = Some(element_pointer.clone());
        node.borrow_mut().keys[0] = Some(key.clone());

        self.key_min = Some(key.clone());
        self.key_max = Some(key.clone());
        self.element_min = Some(element_pointer.clone());
        self.element_max = Some(element_pointer.clone());
        node.borrow_mut().size = 1;

        element_pointer
    }

    fn split_root(&mut self) -> Rc<RefCell<Node<Key, ORDER>>> {
        let new_root = Rc::new(RefCell::new(Node::new(false, None)));
        let old_root = self.root.clone();
        self.root = new_root;
        old_root.borrow_mut().parent = Some(Rc::downgrade(&self.root));
        self.root.borrow_mut().children[0] = Some(old_root);
        Node::split_child(&self.root, 0);
        self.root.clone()
    }

    fn set_extrema(&mut self, element: &Rc<RefCell<Element<Key, ORDER>>>) {
        let key = &element.borrow().key;
        let key_min = &self.key_min;
        let key_max = &self.key_max;
        let mut should_update_weights = false;
        if key_min.is_none() != key_max.is_none() {
            panic!("inconsistent extremas: key_min.is_none() != key_max.is_none()")
        } else if self.key_min.is_none() || self.key_max.is_none() {
            self.key_min = Some(key.clone());
            self.key_max = Some(key.clone());
            self.element_min = Some(element.clone());
            self.element_max = Some(element.clone());
            should_update_weights = true;
        } else {
            if key < key_min.as_ref().unwrap() {
                self.key_min = Some(key.clone());
                self.element_min = Some(element.clone());
                should_update_weights = true;
            }
            if key > key_max.as_ref().unwrap() {
                self.key_max = Some(key.clone());
                self.element_max = Some(element.clone());
                should_update_weights = true;
            }   
        }

        if should_update_weights {
            let range = self.key_min.as_ref().unwrap().distance(self.key_max.as_ref().unwrap());
            self.update_elements_weights(range as f32);
        }
    }

    fn update_elements_weights(&mut self, range: f32) {
        let mut element = match &self.element_min {
            Some(e) => e.clone(),
            None => return
        };
        loop {
            match &element.borrow().prev {
                Some(e) => {
                    let element_ptr = element.as_ptr();
                    let prev_element = e.0.upgrade().unwrap().clone();
                    let prev_element_ptr = prev_element.as_ptr();
                    let weight = element.borrow().weight(&*prev_element.borrow(), range);
                    unsafe { (&mut *element_ptr).prev = Some((Rc::downgrade(&prev_element), weight)) };
                    unsafe { (&mut *prev_element_ptr).next = Some((Rc::downgrade(&element), weight)) };
                },
                None => {}
            };

            let next_element= match &element.borrow().next {
                Some(e) => {
                    let element_ptr = element.as_ptr();
                    let next_element = e.0.upgrade().unwrap().clone();
                    let next_element_ptr = next_element.as_ptr();
                    let weight = element.borrow().weight(&*next_element.borrow(), range);
                    unsafe { (&mut *element_ptr).next = Some((Rc::downgrade(&next_element), weight)) };
                    unsafe { (&mut *next_element_ptr).prev = Some((Rc::downgrade(&element), weight)) };
                    next_element
                },
                None => return
            };
            element = next_element;
        }
    }

    pub fn count_elements_unique(&self) -> usize {
        let mut counter = 0usize;
        let mut element = match &self.element_min {
            Some(e) => e.clone(),
            None => return counter
        };
        loop {
            counter += 1;
            let new_element= match &element.borrow().next {
                Some(e) => e.0.upgrade().unwrap().clone(),
                None => return counter
            };
            element = new_element;
        }
    }

    pub fn count_elements_agg(&self) -> usize {
        let mut counter = 0usize;
        let mut element = match &self.element_min {
            Some(e) => e.clone(),
            None => return counter
        };
        loop {
            counter += element.borrow().counter;
            let new_element= match &element.borrow().next {
                Some(e) => e.0.upgrade().unwrap().clone(),
                None => return counter
            };
            element = new_element;
        }
    }
}

impl<'a, Key, const ORDER: usize> IntoIterator for &'a ASAGraph<Key, ORDER> 
where Key: SensorData + 'static, [(); ORDER + 1]: {
    type Item = Rc<RefCell<Element<Key, ORDER>>>;
    type IntoIter = ASAGraphIntoIterator<'a, Key, ORDER>;

    fn into_iter(self) -> Self::IntoIter {
        ASAGraphIntoIterator {
            graph: self,
            index: match self.element_min.clone() {
                Some(element_min) => Some(element_min.clone()),
                None => None
            },
        }
    }
}

pub struct ASAGraphIntoIterator<'a, Key, const ORDER: usize = 25>
where Key: SensorData + 'static, [(); ORDER + 1]: {
    graph: &'a ASAGraph<Key, ORDER>,
    index: Option<Rc<RefCell<Element<Key, ORDER>>>>
}

impl<'a, Key, const ORDER: usize> Iterator for ASAGraphIntoIterator<'a, Key, ORDER> 
where Key: SensorData + 'static, [(); ORDER + 1]: {
    type Item = Rc<RefCell<Element<Key, ORDER>>>;
    fn next(&mut self) -> Option<Rc<RefCell<Element<Key, ORDER>>>> {
        let next_option;
        let result = match self.index.clone() {
            Some(element) => {
                next_option = match &element.borrow().next {
                    Some(next_tuple) => match next_tuple.0.upgrade() {
                        Some(next) => Some(next),
                        None => None
                    },
                    None => None
                };

                Some(element)
            },
            None => return None
        };

        self.index = next_option;

        result
    }
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;
    use std::{ time::Instant };

    use bionet_common::{
        data::DataCategory,
        sensor::Sensor,
        neuron::Neuron
    };

    use super::ASAGraph;
    use super::super::element::Element;

    #[test]
    fn create_empty_graph() {
        ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);
    }

    #[test]
    fn create_100_elements_graph() {
        let mut rng = rand::thread_rng();

        let start = Instant::now();

        let mut graph = Box::new(ASAGraph::<i32, 3>::new("test", DataCategory::Numerical));

        let n = 1_000;
        for _ in 0..n {
            let number: i32 = rng.gen();
            graph.insert(&number);
        }

        let duration = start.elapsed();

        println!("Time elapsed for ASAGraph insertion of {} elements is is: {:?}", n, duration);
    }

    #[test]
    fn print_graph() {
        let mut rng = rand::thread_rng();

        let mut graph = ASAGraph::<i32, 5>::new("test", DataCategory::Numerical);

        for _ in 0..50 {
            let number: i32 = rng.gen_range(1..=20);
            graph.insert(&number);
        }

        graph.print_graph();
    }

    #[test]
    fn insert_3_degree() {
        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);

        for i in 1..=250 {
            graph.insert(&i);
        }

        for i in (150..=500).rev() {
            graph.insert(&i);
        }

        assert_eq!(graph.count_elements_unique(), 500);
        assert_eq!(graph.count_elements_agg(), 601);

        let root_first_key = graph.root.borrow().elements[0].as_ref().unwrap().borrow().key;
        assert_eq!(root_first_key, 128);
        assert_eq!(graph.key_min.unwrap(), 1);
        assert_eq!(graph.element_min.as_ref().unwrap().borrow().key, 1);
        assert_eq!(graph.key_max.unwrap(), 500);
        assert_eq!(graph.element_max.as_ref().unwrap().borrow().key, 500);

        graph.print_graph();
    }

    #[test]
    fn insert_25_degree() {
        let mut graph = ASAGraph::<i32, 25>::new("test", DataCategory::Numerical);

        for i in 1..=250 {
            graph.insert(&i);
        }

        for i in (150..=500).rev() {
            graph.insert(&i);
        }

        assert_eq!(graph.count_elements_unique(), 500);
        assert_eq!(graph.count_elements_agg(), 601);

        let root_first_key = graph.root.borrow().elements[0].as_ref().unwrap().borrow().key;
        assert_eq!(root_first_key, 169);
        assert_eq!(graph.key_min.unwrap(), 1);
        assert_eq!(graph.element_min.as_ref().unwrap().borrow().key, 1);
        assert_eq!(graph.key_max.unwrap(), 500);
        assert_eq!(graph.element_max.as_ref().unwrap().borrow().key, 500);

        graph.print_graph();
    }

    #[test]
    fn search() {
        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);

        let n = 100;
        for i in 0..n {
            graph.insert(&i);
        }

        for i in 0..n {
            let result = graph.search(&i);
            assert!(result.is_some());
            assert_eq!(result.unwrap().borrow().key, i);
        }
        
        assert!(graph.search(&101).is_none());
        assert!(graph.search(&-1).is_none());
    }

    #[test]
    fn test_connections() {
        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);
    
        let n = 50;
        for i in 1..=n {
            graph.insert(&i);
        }

        let mut prev_element;
        let mut current_element = graph.element_min.as_ref().unwrap().clone();
        for i in 1..=n {
            assert_eq!(current_element.borrow().key, i);
            {
                let prev = &current_element.borrow().prev;
                let next = &current_element.borrow().next;
                if i == 1 { 
                    assert!(prev.is_none());
                    assert_eq!(next.as_ref().unwrap().0.upgrade().unwrap().borrow().key, 2);
                } else if i == n {
                    assert_eq!(prev.as_ref().unwrap().0.upgrade().unwrap().borrow().key, n - 1);
                    assert!(next.is_none());
                    break
                } else {
                    assert_eq!(prev.as_ref().unwrap().0.upgrade().unwrap().borrow().key, i - 1);
                    assert_eq!(next.as_ref().unwrap().0.upgrade().unwrap().borrow().key, i + 1);
                }
            }
            prev_element = current_element.clone();
            current_element = prev_element.borrow().next.as_ref().unwrap().0.upgrade().unwrap().clone();
        }
    }

    #[test]
    fn test_connections_rev() {
        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);
    
        let n = 50;
        for i in (1..=n).rev() {
            graph.insert(&i);
        }

        let mut prev_element;
        let mut current_element = graph.element_min.as_ref().unwrap().clone();
        for i in 1..=n {
            assert_eq!(current_element.borrow().key, i);
            {
                let prev = &current_element.borrow().prev;
                let next = &current_element.borrow().next;
                if i == 1 { 
                    assert!(prev.is_none());
                    assert_eq!(next.as_ref().unwrap().0.upgrade().unwrap().borrow().key, 2);
                } else if i == n {
                    assert_eq!(prev.as_ref().unwrap().0.upgrade().unwrap().borrow().key, n - 1);
                    assert!(next.is_none());
                    break
                } else {
                    assert_eq!(prev.as_ref().unwrap().0.upgrade().unwrap().borrow().key, i - 1);
                    assert_eq!(next.as_ref().unwrap().0.upgrade().unwrap().borrow().key, i + 1);
                }
            }
            prev_element = current_element.clone();
            current_element = prev_element.borrow().next.as_ref().unwrap().0.upgrade().unwrap().clone();
        }
    }

    #[test]
    fn iterator_test() {
        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);
        let n = 50;
        for i in (0..=n).rev() { graph.insert(&i); }
        for (i, element) in graph.into_iter().enumerate() {
            assert_eq!(element.borrow().key, i as i32);
        }
        assert_eq!(graph.key_min.unwrap(), 0i32);
    }

    #[test]
    fn sensor() {
        assert_eq!(Element::<i32, 3>::INTERELEMENT_ACTIVATION_THRESHOLD, 0.8f32);

        let mut graph = ASAGraph::<i32, 3>::new("test", DataCategory::Numerical);
        for i in (1..=9).rev() { graph.insert(&i); }
        
        assert_eq!(graph.name(), "test");
        assert_eq!(graph.data_category(), DataCategory::Numerical);

        let neurons = graph.activate(&5, 1.0f32, true, true);
        assert!(neurons.is_ok());
        assert_eq!(neurons.unwrap().len(), 0);
        
        for (i, element) in graph.into_iter().enumerate() {
            let activation = element.borrow().activation();
            match i + 1 {
                1 => assert_eq!(activation, 0.0f32),
                2 => assert_eq!(activation, 0.0f32),
                3 => assert_eq!(activation, 0.765625f32),
                4 => assert_eq!(activation, 0.875f32),
                5 => assert_eq!(activation, 1.0f32),
                6 => assert_eq!(activation, 0.875f32),
                7 => assert_eq!(activation, 0.765625f32),
                8 => assert_eq!(activation, 0.0f32),
                9 => assert_eq!(activation, 0.0f32),
                _ => {}
            };
        }
        let result = graph.deactivate(&4, true, true);
        assert!(result.is_ok());
        for element in graph.into_iter() {
            let activation = element.borrow().activation();
            assert_eq!(activation, 0.0f32)
        }

        let neurons = graph.activate(&5, 1.0f32, true, true);
        assert!(neurons.is_ok());
        for (i, element) in graph.into_iter().enumerate() {
            let activation = element.borrow().activation();
            match i + 1 {
                1 => assert_eq!(activation, 0.0f32),
                2 => assert_eq!(activation, 0.0f32),
                3 => assert_eq!(activation, 0.765625f32),
                4 => assert_eq!(activation, 0.875f32),
                5 => assert_eq!(activation, 1.0f32),
                6 => assert_eq!(activation, 0.875f32),
                7 => assert_eq!(activation, 0.765625f32),
                8 => assert_eq!(activation, 0.0f32),
                9 => assert_eq!(activation, 0.0f32),
                _ => {}
            };
        }
        graph.deactivate_sensor();
        for element in graph.into_iter() {
            let activation = element.borrow().activation();
            assert_eq!(activation, 0.0f32)
        }

        let neurons = graph.activate(&5, 1.0f32, false, false);
        assert!(neurons.is_ok());
        let neurons = graph.activate(&8, 1.0f32, false, false);
        assert!(neurons.is_ok());
        assert_eq!(neurons.unwrap().len(), 0);
        for (i, element) in graph.into_iter().enumerate() {
            let activation = element.borrow().activation();
            match i + 1 {
                1 => assert_eq!(activation, 0.0f32),
                2 => assert_eq!(activation, 0.0f32),
                3 => assert_eq!(activation, 0.0f32),
                4 => assert_eq!(activation, 0.0f32),
                5 => assert_eq!(activation, 1.0f32),
                6 => assert_eq!(activation, 0.0f32),
                7 => assert_eq!(activation, 0.0f32),
                8 => assert_eq!(activation, 1.0f32),
                9 => assert_eq!(activation, 0.0f32),
                _ => {}
            };
        }
        let result = graph.deactivate(&5, false, false);
        assert!(result.is_ok());
        for (i, element) in graph.into_iter().enumerate() {
            let activation = element.borrow().activation();
            let n = i + 1;
            if n == 8 { assert_eq!(activation, 1.0f32) } else { assert_eq!(activation, 0.0f32) }
        }
    }
}