use std::{
    fmt::{ Display, Formatter, Result },
    rc::{ Rc, Weak },
    cell::RefCell
};

use bionet_common::{
    distances::Distance,
    neuron::{ Neuron, NeuronID }
};

#[derive(Clone, Debug)]
pub struct Element<Key, const ORDER: usize>
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]: {
    pub key: Key,
    pub counter: usize,
    pub activation: f32,
    pub parent: Rc<str>,
    pub(crate) next: Option<Weak<RefCell<Element<Key, ORDER>>>>,
    pub(crate) prev: Option<Weak<RefCell<Element<Key, ORDER>>>>,
}

impl<Key, const ORDER: usize> Element<Key, ORDER> 
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]:  {
    pub fn new(key: &Key, parent: &Rc<str>)
    -> Element<Key, ORDER> {
        Element {
            key: key.clone(),
            counter: 1,
            activation: 0.0f32,
            next: None,
            prev: None,
            parent: parent.clone()
        }
    }

    pub fn set_connections(
        element_ptr: &Rc<RefCell<Element<Key, ORDER>>>,
        prev_opt: Option<&Rc<RefCell<Element<Key, ORDER>>>>,
        next_opt: Option<&Rc<RefCell<Element<Key, ORDER>>>>
    ) {
        let mut element = element_ptr.borrow_mut();
        
        if prev_opt.is_some() {
            let prev_ptr = prev_opt.unwrap();
            element.prev = Some(Rc::downgrade(prev_ptr));
            prev_ptr.borrow_mut().next = Some(Rc::downgrade(element_ptr));
        } else { 
            element.prev = None; 
        }

        if next_opt.is_some() {
            let next_ptr = next_opt.unwrap();
            element.next = Some(Rc::downgrade(next_ptr));
            next_ptr.borrow_mut().prev = Some(Rc::downgrade(&element_ptr));
        } else { 
            element.next = None; 
        }
    }
}

impl<Key, const ORDER: usize> Neuron for Element<Key, ORDER> 
where Key: Clone + Display + Distance + PartialOrd + PartialEq, [(); ORDER + 1]: {
    fn id(&self) -> NeuronID {
        NeuronID {
            id: Rc::from(self.key.to_string()),
            parent_id: self.parent.clone()
        }
    }

    fn activation(&self) -> f32 { self.activation }

    fn stimulate(
        &mut self, signal: f32, propagate_horizontal: bool, propagate_vertical: bool
    ) -> f32 {
        self.activation += signal;
        self.activation
    }

    fn is_sensor(&self) -> bool { true }

    fn counter(&self) -> usize { self.counter }
}

impl<Key, const ORDER: usize> Display for Element<Key, ORDER> 
where Key: Clone + Display + Distance + PartialOrd + PartialEq, [(); ORDER + 1]: {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[{}:{}]", &self.key, &self.counter)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        rc::Rc,
        cell::RefCell
    };

    use super::super::{
        element::Element,
        graph::ASAGraph
    };

    #[test]
    fn set_connections() {
        let graph = Rc::new(RefCell::new(ASAGraph::<i32, 3>::new("test")));
        let graph_name = &graph.borrow().name;

        let element_1_ptr: Rc<RefCell<Element<i32, 3>>> = Rc::new(RefCell::new(Element::new(&1, graph_name)));
        let element_2_ptr: Rc<RefCell<Element<i32, 3>>> = Rc::new(RefCell::new(Element::new(&2, graph_name)));
        let element_3_ptr: Rc<RefCell<Element<i32, 3>>> = Rc::new(RefCell::new(Element::new(&3, graph_name)));

        assert!(element_1_ptr.borrow().prev.is_none());
        assert!(element_1_ptr.borrow().next.is_none());
        assert!(element_2_ptr.borrow().prev.is_none());
        assert!(element_2_ptr.borrow().next.is_none());
        assert!(element_3_ptr.borrow().prev.is_none());
        assert!(element_3_ptr.borrow().next.is_none());
        
        Element::set_connections(&element_2_ptr, Some(&element_1_ptr), None);

        assert!(element_1_ptr.borrow().prev.is_none());
        assert_eq!(
            element_1_ptr.borrow().next.as_ref().unwrap().upgrade().unwrap().borrow().key,
            element_2_ptr.borrow().key
        );
        assert!(element_2_ptr.borrow().next.is_none());
        assert!(element_3_ptr.borrow().prev.is_none());
        assert!(element_3_ptr.borrow().next.is_none());

        Element::set_connections(&element_2_ptr, None, Some(&element_3_ptr));

        assert!(element_1_ptr.borrow().prev.is_none());
        assert_eq!(
            element_1_ptr.borrow().next.as_ref().unwrap().upgrade().unwrap().borrow().key,
            element_2_ptr.borrow().key
        );
        assert!(element_2_ptr.borrow().prev.is_none());
        assert_eq!(
            element_2_ptr.borrow().next.as_ref().unwrap().upgrade().unwrap().borrow().key,
            element_3_ptr.borrow().key
        );
        assert_eq!(
            element_3_ptr.borrow().prev.as_ref().unwrap().upgrade().unwrap().borrow().key, 
            element_2_ptr.borrow().key
        );
        assert!(element_3_ptr.borrow().next.is_none());

        Element::set_connections(&element_1_ptr, None, None);
        Element::set_connections(&element_2_ptr, None, None);
        Element::set_connections(&element_3_ptr, None, None);

        assert!(element_1_ptr.borrow().prev.is_none());
        assert!(element_1_ptr.borrow().next.is_none());
        assert!(element_2_ptr.borrow().prev.is_none());
        assert!(element_2_ptr.borrow().next.is_none());
        assert!(element_3_ptr.borrow().prev.is_none());
        assert!(element_3_ptr.borrow().next.is_none());
    }

    #[test]
    fn parent_name() {
        let graph = Rc::new(RefCell::new(ASAGraph::<i32, 3>::new("test")));
        let graph_name_ptr = &graph.borrow().name;

        let element_1_ptr: Rc<RefCell<Element<i32, 3>>> = Rc::new(RefCell::new(Element::new(&1, graph_name_ptr)));
        let parent_name = &*element_1_ptr.borrow().parent;
        assert_eq!(parent_name, "test");
    }
}