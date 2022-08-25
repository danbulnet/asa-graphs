use std::{
    fmt::Display,
    ptr,
    mem,
    rc::{ Rc, Weak },
    cell::{ RefCell, Ref, RefMut }
};

use bionet_common::{
    sensor::Sensor,
    distances::Distance
};

use crate::{
    element::Element,
    node::Node
};

#[derive(Clone, Debug)]
pub struct ASAGraph<Key, const ORDER: usize = 25>
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]: {
    pub name: String,
    pub(crate) root: Rc<RefCell<Node<Key, ORDER>>>,
    pub(crate) element_min: Option<Rc<RefCell<Element<Key, ORDER>>>>,
    pub(crate) element_max: Option<Rc<RefCell<Element<Key, ORDER>>>>,
    pub key_min: Option<Key>,
    pub key_max: Option<Key>
}

// impl<Key, const ORDER: usize> Sensor for ASAGraph<Key, ORDER> 
// where Key: Clone + Display + PartialOrd + PartialEq + Distance {
//     type ElementType = Element<Key, ORDER>;
//     type DataType = Key;

//     fn name(&self) -> &str { &self.name }

//     fn new(name: &str) -> ASAGraph<Key, ORDER> { Self::new(name) }

//     fn search(&self, key: &Key) -> Option<&Element<Key, ORDER>> { self.search(key) }

//     fn insert(&mut self, key: &Key) -> &Element<Key, ORDER> { self.insert(key) }
// }

impl<Key, const ORDER: usize> ASAGraph<Key, ORDER> 
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]: {
    pub fn new(name: &str) -> ASAGraph<Key, ORDER> {
        ASAGraph {
            name: name.to_string(),
            root: Rc::new(RefCell::new(Node::<Key, ORDER>::new(true, None))),
            element_min: None,
            element_max: None,
            key_min: None,
            key_max: None
        }
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
                    // node = node.children[index + 1].as_ref().unwrap().borrow();
                    index += 1;
                }
            }
            let node_ptr = node.children[index].as_ref().unwrap();
            unsafe { node = node_ptr.try_borrow_unguarded().unwrap() };
        }
    }

//     pub fn insert(&mut self, key: &Key) -> &Element<Key, ORDER> {
//         let self_ptr = self as *mut ASAGraph::<Key, ORDER>;
//         let mut node = &mut self.root;
//         let mut node_ptr = node as *mut Node<Key, ORDER>;

//         if node.size == 0 { 
//             return unsafe { (&mut *self_ptr).insert_first_element(node, key) }
//         }

//         let (key_min, key_max) = unsafe {
//             (&mut *self_ptr).extreme_keys().unwrap_or_else(|| {
//                 panic!("element_min / element_min must not be nullptr")
//             })
//         };

//         if node.size == Node::<Key, ORDER>::MAX_KEYS {
//             unsafe { node = (&mut *self_ptr).split_root() }
//         }

//         loop {
//             let node_insert_result;
//             unsafe {
//                 println!("maxdist {} mindist {}", key.distance(key_max), key.distance(key_min));
//                 println!("key_max {} key_min {}", key_max, key_min);
//                 if key.distance(key_max) > key.distance(key_min) {
//                     println!("left");
//                     node_insert_result = (&mut *node_ptr).insert_existing_key(key, true);
//                 } else {
//                     println!("right");
//                     node_insert_result = (&mut *node_ptr).insert_existing_key(key, false);
//                 };
//             }
//             if let Some(el) = node_insert_result.0 { return el }
//             let mut index = node_insert_result.1;
    
//             if node.is_leaf {
//                 println!("key_min {key_min}, key_max {key_max}");
//                 let element = unsafe {
//                     (&mut *node_ptr).insert_key_leaf(key, self_ptr)
//                 };
//                 println!("key_min {key_min}, key_max {key_max}");
//                 unsafe { (&mut *self_ptr).set_extrema(
//                     element as *mut Element<Key, ORDER>, key_min, key_max
//                 ); }
//                 return element
//             } else {
//                 if node.children[index].as_ref().unwrap().size == Node::<Key, ORDER>::MAX_KEYS {
//                     node.split_child(index);
//                     if key > &node.elements[index].as_ref().unwrap().key { index += 1 }
//                 }
//                 node = node.children[index].as_mut().unwrap();
//                 node_ptr = node as *mut Node<Key, ORDER>;
//             }
//         }
//     }

//     pub fn print_tree(&self) {
//         let mut height = 0;
//         let mut node = &self.root;
//         let mut queue: Vec<Vec<&Node<Key, ORDER>>> = vec![vec![]];
//         queue[0].push(node);

//         loop {
//             queue.push(vec![]);
//             for i in 0..(queue[height].len()) {
//                 node = queue[height][i];
//                 print!("||");
//                 for j in 0..(node.size) {
//                     let element = &node.elements[j].as_ref().unwrap();
//                     print!("{}:{}|", &element.key, element.counter);
//                     if !node.is_leaf {
//                         queue[height + 1].push(node.children[j].as_ref().unwrap());
//                     }
//                 }
//                 if !node.is_leaf {
//                     queue[height + 1].push(node.children[node.size].as_ref().unwrap());
//                 }
//                 print!("| ");
//             }
//             if queue.last().unwrap().len() > 0 {
//                 height += 1;
//                 println!("");
//             } else {
//                 println!("");
//                 return
//             }
//         }
//     }

    fn extreme_keys<'a>(&'a self) -> Option<(&'a Key, &'a Key)> {
        if self.key_min.is_none() || self.key_max.is_none() { return None }
        let key_min =  self.key_min.as_ref().unwrap();
        let key_max =  self.key_max.as_ref().unwrap();
        Some((key_min, key_max))
    }

//     fn insert_first_element(
//         &mut self, node: &mut Node<Key, ORDER>,  key: &Key
//     ) -> &Element<Key, ORDER> {
//         node.elements[0] = Some(
//             Element::<Key, ORDER>::new(key, self as *mut ASAGraph<Key, ORDER>)
//         );
//         let element_pointer = 
//             node.elements[0].as_mut().unwrap() as *mut Element<Key, ORDER>;
//         self.element_min = element_pointer;
//         self.element_max = element_pointer;
//         node.size = 1;

//         return unsafe { &*element_pointer }
//     }

//     fn split_root(&mut self) -> &mut Node<Key, ORDER> {
//         let mut root_placeholder = Node::<Key, ORDER>::new(false);
//         // mem::swap(&mut root_placeholder, &mut self.root);
//         root_placeholder = mem::replace(&mut self.root, root_placeholder);
//         let root = &mut self.root;
//         root.children[0] = Some(root_placeholder);
//         root.split_child(0);
//         root
//         // let root_old = self.root.clone();
//         // self.root = Node::<Key, ORDER>::new(false);
//         // self.root.children[0] = Some(root_old);
//         // self.root.split_child(0);
//         // &mut self.root
//     }

//     fn set_extrema(
//         &mut self, element: *mut Element<Key, ORDER>, key_min: &Key, key_max: &Key
//     ) {
//         let key = unsafe { &(&*element).key };

//         println!("set_extrema: k:{key} min:{key_min} max:{key_max}");
//         if key < key_min {
//             println!("+");
//             self.element_min = element as *mut Element<Key, ORDER>;
//         }
//         if key > key_max {
//             println!("-");
//             self.element_max = element as *mut Element<Key, ORDER>;
//         }
//     }
}

// #[cfg(test)]
// pub mod tests {
//     use rand::Rng;
//     use std::{ io::{self, Write}, time::Instant };
    
//     use super::ASAGraph;

//     #[test]
//     fn create_empty_graph() {
//         ASAGraph::<i32, 3>::new("test");
//     }

//     #[test]
//     fn create_100_elements_graph() {
//         let mut rng = rand::thread_rng();

//         let start = Instant::now();

//         let mut graph = ASAGraph::<i32, 3>::new("test");

//         let n = 1_000_000;
//         for _ in 0..n {
//             let number: i32 = rng.gen();
//             graph.insert(&number);
//         }

//         let duration = start.elapsed();

//         println!("Time elapsed for ASAGraph insertion of {} elements is is: {:?}", n, duration);
//     }

//     #[test]
//     fn print_tree() {
//         let mut rng = rand::thread_rng();

//         let mut graph = ASAGraph::<i32, 3>::new("test");

//         for _ in 0..50 {
//             let number: i32 = rng.gen_range(1..=20);
//             graph.insert(&number);
//         }

//         io::stdout().flush().unwrap();
//         graph.print_tree();
//         io::stdout().flush().unwrap();
//     }

//     #[test]
//     fn search() {
//         let mut graph = ASAGraph::<i32, 3>::new("test");

//         let n = 100;
//         for i in 0..n {
//             graph.insert(&i);
//         }

//         for i in 0..n {
//             assert!(graph.search(&i).is_some());
//         }
        
//         assert!(graph.search(&101).is_none());
//         assert!(graph.search(&-1).is_none());
//     }

    // #[test]
    // fn test_connections() {
    //     let mut prev_element;
    //     let mut current_element = left_left_child.borrow().elements[0].as_ref().unwrap().clone();
    //     for i in 1..=7 {
    //         println!("\n{i}-th element: {}", current_element.borrow());
    //         assert_eq!(current_element.borrow().key, i);
    //         {
    //             let prev = &current_element.borrow().prev;
    //             let next = &current_element.borrow().next;
    //             println!("\n{i}-th next element: {}", next.as_ref().unwrap().borrow());
    //             if i == 1 { 
    //                 assert!(prev.is_none());
    //                 assert_eq!(next.as_ref().unwrap().borrow().key, 2);
    //             } else if i == 7 {
    //                 assert_eq!(prev.as_ref().unwrap().upgrade().unwrap().borrow().key, 6);
    //                 assert!(next.is_none());
    //                 break
    //             } else {
    //                 assert_eq!(prev.as_ref().unwrap().upgrade().unwrap().borrow().key, i - 1);
    //                 assert_eq!(next.as_ref().unwrap().borrow().key, i + 1);
    //             }
    //         }
    //         prev_element = current_element.clone();
    //         current_element = prev_element.borrow().next.as_ref().unwrap().clone();
    //     }
    // }
// }