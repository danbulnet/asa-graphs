use std::{
    fmt::{ Display, Formatter, Result },
    rc::{ Rc, Weak },
    cell::RefCell,
    ptr
};

use bionet_common::{
    distances::Distance
};

use crate::{
    graph::ASAGraph
};

#[derive(Clone, Debug)]
pub struct Element<Key, const ORDER: usize>
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]: {
    pub key: Key,
    pub counter: usize,
    pub(crate) next: Option<Rc<RefCell<Element<Key, ORDER>>>>,
    pub(crate) prev: Option<Weak<RefCell<Element<Key, ORDER>>>>,
    pub(crate) parent: *mut ASAGraph<Key, ORDER>
}

impl<Key, const ORDER: usize> Element<Key, ORDER> 
where Key: Clone + Display + PartialOrd + PartialEq + Distance, [(); ORDER + 1]:  {
    pub fn new(key: &Key, parent: *mut ASAGraph<Key, ORDER>)
    -> Element<Key, ORDER> {
        Element {
            key: key.clone(),
            next: None,
            prev: None,
            counter: 1,
            parent
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
            prev_ptr.borrow_mut().next = Some(element_ptr.clone());
        } else { 
            element.prev = None; 
        }

        if next_opt.is_some() {
            let next_ptr = next_opt.unwrap();
            element.next = Some(next_ptr.clone());
            next_ptr.borrow_mut().prev = Some(Rc::downgrade(&element_ptr));
        } else { 
            element.next = None; 
        }
    }

    pub unsafe fn get_parent_ptr(&self) -> Option<*mut ASAGraph<Key, ORDER>> {
        if self.parent.is_null() { None } else { Some(self.parent) }
    }
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

    use bionet_common::sensor::Sensor;

    use crate::{
        element::Element,
        graph::ASAGraph
    };

    #[test]
    fn set_connections() {
        let graph = Rc::new(RefCell::new(ASAGraph::<i32, 3>::new("test")));
        let graph_ptr = &mut *graph.borrow_mut() as *mut ASAGraph<i32, 3>;

        let element_1_ptr = Rc::new(RefCell::new(Element::new(&1, graph_ptr)));
        let element_2_ptr = Rc::new(RefCell::new(Element::new(&2, graph_ptr)));
        let element_3_ptr = Rc::new(RefCell::new(Element::new(&3, graph_ptr)));

        assert!(element_1_ptr.borrow().prev.is_none());
        assert!(element_1_ptr.borrow().next.is_none());
        assert!(element_2_ptr.borrow().prev.is_none());
        assert!(element_2_ptr.borrow().next.is_none());
        assert!(element_3_ptr.borrow().prev.is_none());
        assert!(element_3_ptr.borrow().next.is_none());
        
        Element::set_connections(&element_2_ptr, Some(&element_1_ptr), None);

        assert!(element_1_ptr.borrow().prev.is_none());
        assert_eq!(
            element_1_ptr.borrow().next.as_ref().unwrap().borrow().key,
            element_2_ptr.borrow().key
        );
        assert!(element_2_ptr.borrow().next.is_none());
        assert!(element_3_ptr.borrow().prev.is_none());
        assert!(element_3_ptr.borrow().next.is_none());

        Element::set_connections(&element_2_ptr, None, Some(&element_3_ptr));

        assert!(element_1_ptr.borrow().prev.is_none());
        assert_eq!(
            element_1_ptr.borrow().next.as_ref().unwrap().borrow().key,
            element_2_ptr.borrow().key
        );
        assert!(element_2_ptr.borrow().prev.is_none());
        assert_eq!(
            element_2_ptr.borrow().next.as_ref().unwrap().borrow().key,
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
        let graph_ptr = &mut *graph.borrow_mut() as *mut ASAGraph<i32, 3>;

        let element_1_ptr = Rc::new(RefCell::new(Element::new(&1, graph_ptr)));
        let parent_ptr = unsafe { element_1_ptr.borrow().get_parent_ptr() };
        let parent_name = unsafe { (&*parent_ptr.unwrap()).name.clone() };
        assert_eq!(parent_name, "test");
    }
}