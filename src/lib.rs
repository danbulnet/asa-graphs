#![allow(incomplete_features)] // TODO: check if stable versions allows for const generic parameters not only standalone in constexpr
#![feature(generic_const_exprs)] // TODO: check if stable versions allows for const generic parameters not only standalone in constexpr

#![allow(dead_code)] // TODO: remove after dev is done

pub mod graph;
mod node;
mod element;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
