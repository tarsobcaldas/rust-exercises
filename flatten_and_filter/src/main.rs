use std::collections::HashSet;

fn flatten_and_filter(list: Vec<Vec<u32>>) -> Vec<u32> {
    let mut set = HashSet::new();
    list.into_iter()
        .flatten()
        .filter(|&x| (x % 2 == 0 || x % 3 == 0) && set.insert(x))
        .collect()
}

fn main() {
    let list = vec![
        vec![1, 27, 38, 17, 34],
        vec![5, 6, 111, 23, 12, 57],
        vec![7, 9, 13, 15, 19, 21],
    ];
    let result = flatten_and_filter(list);
    println!("{:?}", result);
}
