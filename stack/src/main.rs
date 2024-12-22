use std::any::Any;

struct Stack {
    elements: Vec<Box<dyn Any>>,
}

impl Stack {
    fn new() -> Self {
        Stack {
            elements: Vec::new(),
        }
    }

    fn push<T: 'static>(&mut self, item: T) {
        self.elements.push(Box::new(item));
    }

    fn pop(&mut self) -> Option<Box<dyn Any>> {
        self.elements.pop()
    }

    fn peek(&self) -> Option<&Box<dyn Any>> {
        self.elements.last()
    }

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

fn main() {
    let mut stack = Stack::new();

    stack.push(1);
    stack.push(66.32);
    stack.push("Hello, World!");
    stack.push(vec![1, 2, 3, 4, 5]);
    stack.push(String::from("Rust"));

    println!("Stack length: {}", stack.len());

    if let Some(top) = stack.peek() {
        if let Some(value) = top.downcast_ref::<i32>() {
            println!("Top element is an i32: {}", value);
        } else if let Some(value) = top.downcast_ref::<f64>() {
            println!("Top element is an f64: {}", value);
        } else if let Some(value) = top.downcast_ref::<String>() {
            println!("Top element is a String: {}", value);
        } else {
            println!("Top element is of an unknown type");
        }
    } else {
        println!("The stack is empty");
    }

    while let Some(top) = stack.pop() {
        if let Some(value) = top.downcast_ref::<i32>() {
            println!("Popping i32: {}", value);
        } else if let Some(value) = top.downcast_ref::<f64>() {
            println!("Popping f64: {}", value);
        } else if let Some(value) = top.downcast_ref::<&str>() {
            println!("Popping &str: {}", value);
        } else if let Some(value) = top.downcast_ref::<Vec<i32>>() {
            println!("Popping Vec<i32>: {:?}", value);
        } else if let Some(value) = top.downcast_ref::<String>() {
            println!("Popping String: {}", value);
        } else {
            println!("Unknown type");
        }
    }

    println!("Stack is empty: {}", stack.is_empty());
}
