use rand::Rng;
use std::{
    fmt::{self, Debug, Display},
    io::{self, stdin, stdout, Write},
};

enum ErrorKind {
    Io(io::Error),
    ParseInt,
    EmptyInput,
}

impl ErrorKind {
    fn msg(&self) -> String {
        match self {
            ErrorKind::Io(e) => format!("I/O error: {}", e),
            ErrorKind::ParseInt => "Failed to parse an integer".to_string(),
            ErrorKind::EmptyInput => "Empty input".to_string(),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg())
    }
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> Self {
        ErrorKind::Io(err)
    }
}

fn partition<T: Ord>(v: &mut [T], low: usize, high: usize, pivot: usize) -> usize {
    v.swap(pivot, high);
    let mut store_index = low;
    for i in low..high {
        if v[i] < v[high] {
            v.swap(i, store_index);
            store_index += 1;
        }
    }
    v.swap(store_index, high);
    store_index
}

fn quicksort_base<T: Ord>(v: &mut [T], low: usize, high: usize) {
    if low < high {
        let pivot = (low + high) / 2;
        let pivot_index = partition(v, low, high, pivot);
        if pivot_index > 0 {
            quicksort_base(v, low, pivot_index - 1);
        }
        quicksort_base(v, pivot + 1, high);
    }
}

fn quicksort<T: Ord>(v: &mut [T]) {
    let len = v.len();
    if v.len() <= 1 {
        return;
    }
    quicksort_base(v, 0, len - 1);
}

fn bubble_sort<T: Ord>(v: &mut [T]) {
    let len = v.len();
    for i in 0..len {
        for j in 0..len - i - 1 {
            if v[j] > v[j + 1] {
                v.swap(j, j + 1);
            }
        }
    }
}

fn selection_sort<T: Ord>(v: &mut [T]) {
    let len = v.len();
    for i in 0..len {
        let mut min = i;
        for j in i + 1..len {
            if v[j] < v[min] {
                min = j;
            }
        }
        v.swap(i, min);
    }
}

fn merge_sort<T: Ord + Clone>(v: &mut [T]) {
    fn merge<T: Ord + Clone>(v: &mut [T], low: usize, mid: usize, high: usize) {
        let left = v[low..mid].to_vec();
        let right = v[mid..high].to_vec();
        let mut i = 0;
        let mut j = 0;
        let mut k = low;
        while i < left.len() && j < right.len() {
            if left[i] <= right[j] {
                v[k] = left[i].clone();
                i += 1;
            } else {
                v[k] = right[j].clone();
                j += 1;
            }
            k += 1;
        }
        while i < left.len() {
            v[k] = left[i].clone();
            i += 1;
            k += 1;
        }
        while j < right.len() {
            v[k] = right[j].clone();
            j += 1;
            k += 1;
        }
    }

    fn merge_sort_base<T: Ord + Clone>(v: &mut [T], low: usize, high: usize) {
        if low < high - 1 {
            let mid = (low + high) / 2;
            merge_sort_base(v, low, mid);
            merge_sort_base(v, mid, high);
            merge(v, low, mid, high);
        }
    }

    let len = v.len();
    merge_sort_base(v, 0, len);
}

fn insertion_sort<T: Ord>(v: &mut [T]) {
    let len = v.len();
    for i in 1..len {
        let mut j = i;
        while j > 0 && v[j] < v[j - 1] {
            v.swap(j, j - 1);
            j -= 1;
        }
    }
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[allow(dead_code)]
    fn generate_random_sequence() -> Vec<i32> {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        let mut nums: Vec<i32> = (-1000..1000).collect();
        nums.shuffle(&mut rng);
        nums
    }

    #[test]
    fn test_int_quicksort() {
        let mut nums = generate_random_sequence();
        let mut v1 = nums.clone();
        v1.sort();
        quicksort(&mut nums);
        assert_eq!(nums, v1);
    }

    #[test]
    fn test_int_bubblesort() {
        let mut nums = generate_random_sequence();
        let mut v1 = nums.clone();
        v1.sort();
        bubble_sort(&mut nums);
        assert_eq!(nums, v1);
    }

    #[test]
    fn test_int_selectionsort() {
        let mut nums = generate_random_sequence();
        let mut v1 = nums.clone();
        v1.sort();
        selection_sort(&mut nums);
        assert_eq!(nums, v1);
    }

    #[test]
    fn test_int_insertionsort() {
        let mut nums = generate_random_sequence();
        let mut v1 = nums.clone();
        v1.sort();
        insertion_sort(&mut nums);
        assert_eq!(nums, v1);
    }

    #[test]
    fn test_int_mergesort() {
        let mut nums = generate_random_sequence();
        let mut v1 = nums.clone();
        v1.sort();
        merge_sort(&mut nums);
        assert_eq!(nums, v1);
    }
}

fn read_vec<T: Ord + std::str::FromStr>(vec: &str) -> Result<Vec<T>, ErrorKind> {
    if vec.is_empty() {
        return Err(ErrorKind::EmptyInput);
    }
    vec.split_whitespace()
        .map(|s| s.parse::<T>().map_err(|_| ErrorKind::ParseInt))
        .collect()
}

fn prompt() -> Result<Vec<i32>, ErrorKind> {
    print!("Generate a random vector of integers or input one separated by spaces: ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).map_err(ErrorKind::Io)?;
    match read_vec::<i32>(&buf) {
        Ok(v) => {
            if v.len() == 1 {
                let mut rng = rand::thread_rng();
                let mut nums = Vec::new();

                for _ in 0..v[0] {
                    nums.push(rng.gen_range(-100..100));
                }
                Ok(nums)
            } else {
                Ok(v)
            }
        }
        Err(e) => Err(e),
    }
}

fn time<T: Ord + Debug>(
    v: &mut Vec<T>,
    func: &dyn Fn(&mut [T]),
    func_name: &str,
) -> std::time::Duration {
    let start = std::time::Instant::now();
    func(v);
    let elapsed = start.elapsed();
    println!("Sorted: {:?} in {:?} by {}", v, elapsed, func_name);
    elapsed
}

fn main() {
    match prompt() {
        Ok(v) => {
            let bubble_time = time(&mut v.clone(), &bubble_sort, "Bubble Sort");
            let insertion_time = time(&mut v.clone(), &insertion_sort, "Insertion Sort");
            let selection_time = time(&mut v.clone(), &selection_sort, "Selection Sort");
            let merge_time = time(&mut v.clone(), &merge_sort, "Merge Sort");
            let quick_time = time(&mut v.clone(), &quicksort, "Quick Sort");
            println!("Timings:\nBubble Sort: {:?}\nSelection Sort: {:?}\nInsertion Sort: {:?}\nQuick Sort: {:?}\nMerge Sort: {:?}",
                bubble_time, selection_time, insertion_time, quick_time, merge_time
            );
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
