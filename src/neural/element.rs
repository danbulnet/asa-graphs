use std::{
    fmt::{ Display, Formatter, Result as FmtResult },
    rc::{ Rc, Weak },
    cell::RefCell
};

use bionet_common::{
    distances::Distance,
    neuron::{ Neuron, NeuronConnect, NeuronID },
    connection::{ 
        Connection, 
        ConnectionKind,
        defining_connection::DefiningConnection
    }
};

pub struct Element<Key, const ORDER: usize>
where Key: Clone + Display + PartialOrd + PartialEq + Distance + 'static, [(); ORDER + 1]: {
    pub key: Key,
    pub counter: usize,
    pub activation: f32,
    pub parent: Rc<str>,
    pub(crate) self_ptr: Weak<RefCell<Element<Key, ORDER>>>,
    pub(crate) next: Option<Weak<RefCell<Element<Key, ORDER>>>>,
    pub(crate) prev: Option<Weak<RefCell<Element<Key, ORDER>>>>,
    pub(crate) definitions: Vec<Rc<RefCell<DefiningConnection<Self, dyn Neuron>>>>,
}

impl<Key, const ORDER: usize> Element<Key, ORDER> 
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]:  {
    pub fn new(key: &Key, parent: &Rc<str>)
    -> Rc<RefCell<Element<Key, ORDER>>> {
        let element_ptr = Rc::new(
            RefCell::new(
                Element {
                    key: key.clone(),
                    counter: 1,
                    activation: 0.0f32,
                    parent: parent.clone(),
                    self_ptr: Weak::new(), 
                    next: None,
                    prev: None,
                    definitions: Vec::new()
                }
            )
        );

        element_ptr.borrow_mut().self_ptr = Rc::downgrade(&element_ptr);
        element_ptr
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
where Key: Clone + Display + Distance + PartialOrd + PartialEq + 'static, [(); ORDER + 1]: {
    fn id(&self) -> NeuronID {
        NeuronID {
            id: Rc::from(self.key.to_string()),
            parent_id: self.parent.clone()
        }
    }

    fn activation(&self) -> f32 { self.activation }

    fn is_sensor(&self) -> bool { true }

    fn counter(&self) -> usize { self.counter }

    fn activate(
        &mut self, signal: f32, propagate_horizontal: bool, propagate_vertical: bool
    ) -> Vec<Rc<RefCell<dyn Neuron>>> {
        self.activation += signal;

        let mut activated_neurons = Vec::new();

        if propagate_vertical {
            for e in &self.definitions {
                let to = e.borrow().to().clone();
                let is_sensor = to.borrow().is_sensor();
                to.borrow_mut().activate(self.activation, propagate_horizontal, !is_sensor);
                activated_neurons.push(to);
            }
        }

        activated_neurons
    }

    fn explain(&mut self) -> Vec<Rc<RefCell<dyn Neuron>>> { Vec::new() }

    fn deactivate(&mut self, propagate_horizontal: bool, propagate_vertical: bool) {
        self.activation = 0.0f32;

        if propagate_vertical {
            for e in &self.definitions {
                let to = e.borrow().to().clone();
                let is_sensor = to.borrow().is_sensor();
                to.borrow_mut().deactivate(propagate_horizontal, !is_sensor);
            }
        }
    }
}

impl<Key, const ORDER: usize> NeuronConnect for Element<Key, ORDER> 
where Key: Clone + Display + Distance + PartialOrd + PartialEq + 'static, [(); ORDER + 1]: {
    fn connect(
        &mut self, to: Rc<RefCell<dyn Neuron>>, kind: ConnectionKind
    ) -> Result<Rc<RefCell<dyn Connection<From = Self, To = dyn Neuron>>>, String> {
        match kind {
            ConnectionKind::Defining => {
                let connection = Rc::new(RefCell::new(DefiningConnection::new(
                    self.self_ptr.upgrade().unwrap(), 
                    to
                )));

                self.definitions.push(connection.clone());

                Ok(connection)
            },
            _ => Err("only explanatory connection can be created fo asa-graphs element".to_string())
        }
    }
}

impl<Key, const ORDER: usize> Display for Element<Key, ORDER> 
where Key: Clone + Display + Distance + PartialOrd + PartialEq, [(); ORDER + 1]: {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[{}:{}]", &self.key, &self.counter)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        rc::Rc,
        cell::RefCell
    };

    use bionet_common::{
        neuron::{ Neuron, NeuronConnect },
        connection::ConnectionKind
    };

    use super::super::{
        element::Element,
        graph::ASAGraph
    };

    #[test]
    fn set_connections() {
        let graph = Rc::new(RefCell::new(ASAGraph::<i32, 3>::new("test")));
        let graph_name = &graph.borrow().name;

        let element_1_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&1, graph_name);
        let element_2_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&2, graph_name);
        let element_3_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&3, graph_name);

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

        let element_1_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&1, graph_name_ptr);
        let parent_name = &*element_1_ptr.borrow().parent;
        assert_eq!(parent_name, "test");
    }

    #[test]
    fn as_neuron() {
        let graph = Rc::new(RefCell::new(ASAGraph::<i32, 3>::new("test")));
        let graph_name = &graph.borrow().name;

        let element_1_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&1, graph_name);
        let element_2_ptr: Rc<RefCell<Element<i32, 3>>> = Element::new(&2, graph_name);

        let element_1_id = element_1_ptr.borrow().id();
        assert_eq!(element_1_id.id.to_string(), 1.to_string());
        assert_eq!(element_1_id.parent_id.to_string(), graph.borrow().name.to_string());
        let element_2_id = element_2_ptr.borrow().id();
        assert_eq!(element_2_id.id.to_string(),2.to_string());
        assert_eq!(element_2_id.parent_id.to_string(), graph.borrow().name.to_string());

        assert_eq!(element_1_ptr.borrow().is_sensor(), true);

        assert_eq!(element_1_ptr.borrow().activation(), 0.0f32);
        
        let connection = element_1_ptr
            .borrow_mut().connect(element_2_ptr.clone(), ConnectionKind::Defining).unwrap();
        assert!(std::ptr::eq(connection.borrow().from().as_ptr(), element_1_ptr.as_ptr()));
        assert!(std::ptr::eq(connection.borrow().to().as_ptr(), element_2_ptr.as_ptr()));

        let activated = element_1_ptr.borrow_mut().activate(1.0f32, true, true);
        assert_eq!(activated.len(), 1);

        assert_eq!(element_1_ptr.borrow().activation(), 1.0f32);
        assert_eq!(element_2_ptr.borrow().activation(), 1.0f32);
    }
}