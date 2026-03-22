#![warn(clippy::pedantic)]
#![allow(dead_code)]
use crate::math::Semigroup;

#[derive()]
enum Node{
    Value(usize,usize),
}

// a partial order on usize
fn leq(a: usize, b: usize, ng:&Semigroup) -> bool {
    if a <= b {false} else {
        let delta = b-a;
        ng.element(delta)
    }
}

fn graph(numbers:&[usize], ng:&Semigroup)->Vec<Node>{
    // iterate over all pairs (i,j) in numbers where the first is <= the second
    // then check whether leq(i,j,ng)
    // if so, create a (Node,Node) pair and add it to the result
    numbers
        .iter()
        .enumerate()
        .flat_map(|(idx, &i)| {
            // Start the inner iterator from the current index to ensure i <= j
            numbers[idx..].iter().map(move |&j| (i, j))
        }).filter(|(a,b)| leq(*a,*b,ng)).map(|(a,b)| Node::Value(a,b)).collect()
}