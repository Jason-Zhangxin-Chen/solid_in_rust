// sorting.rs - A collection of sorting algorithms with tests

// -----------------------------------------------------------------------------
// 1. Bubble Sort
// -----------------------------------------------------------------------------
pub fn bubble_sort<T: Ord>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in 0..n {
        // use below flag to quit quickly if arr is already ordered.
        let mut swapped = false;

        // on every section, swap the neighbour nodes, let the largest
        // or the smallest item into the end.
        for j in 0..n - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
}

// -----------------------------------------------------------------------------
// 2. Selection Sort
// -----------------------------------------------------------------------------
pub fn selection_sort<T: Ord>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in 0..n {
        // assume i as the min item idx in current iteration,
        let mut min_idx = i;
        for j in (i + 1)..n {
            // update the min item's idx
            if arr[j] < arr[min_idx] {
                min_idx = j;
            }
        }
        // if the min idx changes, swap them.
        if min_idx != i {
            arr.swap(i, min_idx);
        }
    }
}

// -----------------------------------------------------------------------------
// 3. Insertion Sort
// -----------------------------------------------------------------------------
pub fn insertion_sort<T: Ord>(arr: &mut [T]) {
    // extend the section from smallest to the fully space.
    for i in 1..arr.len() {
        let mut j = i;
        // make the 1st section totally ordered,
        // compare current ith item with its previous one,
        // back trace the item one by one until we sort out
        // all.
        while j > 0 && arr[j] < arr[j - 1] {
            arr.swap(j, j - 1);
            j -= 1;
        }
        // pick up next item from the rest into the 1st
        // section, and make them totally ordered again.
    }
}

// -----------------------------------------------------------------------------
// 4. Merge Sort
// -----------------------------------------------------------------------------
pub fn merge_sort<T: Ord + Clone + Default>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    let mid = n / 2;
    let mut left = arr[..mid].to_vec();
    let mut right = arr[mid..].to_vec();

    merge_sort(&mut left);
    merge_sort(&mut right);

    merge(arr, &left, &right);
}

fn merge<T: Ord + Clone>(target: &mut [T], left: &[T], right: &[T]) {
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;

    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            target[k] = left[i].clone();
            i += 1;
        } else {
            target[k] = right[j].clone();
            j += 1;
        }
        k += 1;
    }

    while i < left.len() {
        target[k] = left[i].clone();
        i += 1;
        k += 1;
    }
    while j < right.len() {
        target[k] = right[j].clone();
        j += 1;
        k += 1;
    }
}

// -----------------------------------------------------------------------------
// 5. Quick Sort
// -----------------------------------------------------------------------------
pub fn quick_sort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    // select a pivot, and partition the items smaller than
    // pivot at the lower section, and larger item in the
    // higher section, and return the pivot.
    let pivot = partition(arr);

    // recursively process the lower section in same way.
    quick_sort(&mut arr[..pivot]);
    // recursively process the higher section in same way.
    // higher section should starts from pivot+1, rather
    // than pivot.
    quick_sort(&mut arr[pivot + 1..]);
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let last = len - 1;

    // Median-of-three pivot selection
    // it places the median value item at the end pos.
    let mid = len / 2;
    if arr[mid] < arr[0] {
        arr.swap(0, mid);
    }
    if arr[last] < arr[0] {
        arr.swap(0, last);
    }
    if arr[mid] < arr[last] {
        arr.swap(mid, last);
    }

    // lower section should start from pos 0.
    let mut i = 0;
    for j in 0..last {
        // put the lower half from 0 to pivot(i)
        if arr[j] <= arr[last] {
            arr.swap(i, j);
            i += 1; // update storage slot for next one.
        }
    }
    // swap the pivot value from last to i which is the
    // resolved final pivot pos.
    arr.swap(i, last);
    i
}

// -----------------------------------------------------------------------------
// 6. Heap Sort
// -----------------------------------------------------------------------------
pub fn heap_sort<T: Ord>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in (0..n / 2).rev() {
        heapify(arr, n, i);
    }

    for i in (1..n).rev() {
        arr.swap(0, i);
        heapify(arr, i, 0);
    }
}

fn heapify<T: Ord>(arr: &mut [T], n: usize, i: usize) {
    let mut largest = i;
    let left = 2 * i + 1;
    let right = 2 * i + 2;

    if left < n && arr[left] > arr[largest] {
        largest = left;
    }
    if right < n && arr[right] > arr[largest] {
        largest = right;
    }

    if largest != i {
        arr.swap(i, largest);
        heapify(arr, n, largest);
    }
}


// -----------------------------------------------------------------------------
// 9. Tim Sort (Simplified)
// -----------------------------------------------------------------------------
pub fn tim_sort<T: Ord + Clone>(arr: &mut [T]) {
    const MIN_RUN: usize = 32;
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in (0..n).step_by(MIN_RUN) {
        let end = (i + MIN_RUN).min(n);
        insertion_sort_range(arr, i, end);
    }

    let mut size = MIN_RUN;
    while size < n {
        for left in (0..n).step_by(2 * size) {
            let mid = left + size;
            let right = (left + 2 * size).min(n);
            if mid < right {
                merge_inplace(arr, left, mid, right);
            }
        }
        size *= 2;
    }
}


// -----------------------------------------------------------------------------
// 7. Counting Sort
// -----------------------------------------------------------------------------
pub fn counting_sort(arr: &[usize]) -> Vec<usize> {
    if arr.is_empty() {
        return vec![];
    }

    let max = *arr.iter().max().unwrap();
    let min = *arr.iter().min().unwrap();
    let range = max - min + 1;

    let mut count = vec![0; range];
    for &val in arr {
        count[val - min] += 1;
    }

    for i in 1..range {
        count[i] += count[i - 1];
    }

    let mut output = vec![0; arr.len()];
    for &val in arr.iter().rev() {
        let idx = val - min;
        count[idx] -= 1;
        output[count[idx]] = val;
    }
    output
}

// -----------------------------------------------------------------------------
// 8. Radix Sort (LSD)
// -----------------------------------------------------------------------------
pub fn radix_sort(arr: &mut [u32]) {
    if arr.len() <= 1 {
        return;
    }

    let max = *arr.iter().max().unwrap_or(&0);
    let mut exp = 1;
    let mut buffer = vec![0; arr.len()];

    while max / exp > 0 {
        let mut count = [0; 10];
        for &num in arr.iter() {
            let digit = ((num / exp) % 10) as usize;
            count[digit] += 1;
        }

        for i in 1..10 {
            count[i] += count[i - 1];
        }

        for &num in arr.iter().rev() {
            let digit = ((num / exp) % 10) as usize;
            count[digit] -= 1;
            buffer[count[digit]] = num;
        }

        arr.copy_from_slice(&buffer);
        exp *= 10;
    }
}

fn insertion_sort_range<T: Ord>(arr: &mut [T], start: usize, end: usize) {
    for i in (start + 1)..end {
        let mut j = i;
        while j > start && arr[j] < arr[j - 1] {
            arr.swap(j, j - 1);
            j -= 1;
        }
    }
}

fn merge_inplace<T: Ord + Clone>(arr: &mut [T], left: usize, mid: usize, right: usize) {
    let left_part = arr[left..mid].to_vec();
    let right_part = arr[mid..right].to_vec();

    let mut i = 0;
    let mut j = 0;
    let mut k = left;

    while i < left_part.len() && j < right_part.len() {
        if left_part[i] <= right_part[j] {
            arr[k] = left_part[i].clone();
            i += 1;
        } else {
            arr[k] = right_part[j].clone();
            j += 1;
        }
        k += 1;
    }

    while i < left_part.len() {
        arr[k] = left_part[i].clone();
        i += 1;
        k += 1;
    }
    while j < right_part.len() {
        arr[k] = right_part[j].clone();
        j += 1;
        k += 1;
    }
}

// -----------------------------------------------------------------------------
// Tests (combined)
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn test_bubble_sort() {
        let mut v = vec![5, 1, 4, 2, 8];
        bubble_sort(&mut v);
        assert_eq!(v, vec![1, 2, 4, 5, 8]);

        let mut v = vec![1, 2, 3, 4, 5];
        bubble_sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5]);

        let mut v = vec![5, 4, 3, 2, 1];
        bubble_sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5]);

        let mut v: Vec<i32> = vec![];
        bubble_sort(&mut v);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_selection_sort() {
        let mut v = vec![64, 25, 12, 22, 11];
        selection_sort(&mut v);
        assert_eq!(v, vec![11, 12, 22, 25, 64]);

        let mut v = vec![3, 1, 2, 1, 3];
        selection_sort(&mut v);
        assert_eq!(v, vec![1, 1, 2, 3, 3]);
    }

    #[test]
    fn test_insertion_sort() {
        let mut v = vec![5, 2, 4, 6, 1, 3];
        insertion_sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5, 6]);

        let mut v = vec![2, 1];
        insertion_sort(&mut v);
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn test_merge_sort() {
        let mut v = vec![38, 27, 43, 3, 9, 82, 10];
        merge_sort(&mut v);
        assert_eq!(v, vec![3, 9, 10, 27, 38, 43, 82]);

        let mut v: Vec<i32> = vec![];
        merge_sort(&mut v);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_quick_sort() {
        let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        quick_sort(&mut v);
        assert_eq!(v, vec![1, 1, 2, 3, 4, 5, 6, 9]);

        let mut v = vec![1, 2, 3, 4, 5];
        quick_sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_heap_sort() {
        let mut v = vec![12, 11, 13, 5, 6, 7];
        heap_sort(&mut v);
        assert_eq!(v, vec![5, 6, 7, 11, 12, 13]);

        let mut v = vec![5, 4, 3, 2, 1];
        heap_sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_counting_sort() {
        let v = vec![4, 2, 2, 8, 3, 3, 1];
        let sorted = counting_sort(&v);
        assert_eq!(sorted, vec![1, 2, 2, 3, 3, 4, 8]);
    }

    #[test]
    fn test_radix_sort() {
        let mut v = vec![170, 45, 75, 90, 802, 24, 2, 66];
        radix_sort(&mut v);
        assert_eq!(v, vec![2, 24, 45, 66, 75, 90, 170, 802]);

        let mut v = vec![1000, 234, 56, 7890, 1];
        radix_sort(&mut v);
        assert_eq!(v, vec![1, 56, 234, 1000, 7890]);
    }

    #[test]
    fn test_tim_sort() {
        let mut v = vec![5, 21, 7, 23, 19, 10, 12, 4, 8];
        tim_sort(&mut v);
        assert_eq!(v, vec![4, 5, 7, 8, 10, 12, 19, 21, 23]);

        // Large random test (requires rand crate in Cargo.toml when used in a project)
        let mut rng = rand::rng();
        let mut v: Vec<i32> = (0..100).collect();
        v.shuffle(&mut rng);
        let mut expected = v.clone();
        expected.sort();
        tim_sort(&mut v);
        assert_eq!(v, expected);
    }
}