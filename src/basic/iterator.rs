// A simple container: a wrapper around a Vec
#[derive(Debug)]
struct MyVec<T>(Vec<T>);

// --- Owned iterator ---
struct MyIntoIter<T>(std::vec::IntoIter<T>);

impl<T> Iterator for MyIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = MyIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        MyIntoIter(self.0.into_iter())
    }
}

// --- Borrowed iterator ---
struct MyIter<'a, T>(std::slice::Iter<'a, T>);

impl<'a, T> Iterator for MyIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T> IntoIterator for &'a MyVec<T> {
    type Item = &'a T;
    type IntoIter = MyIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        MyIter(self.0.iter())
    }
}

// --- Mutable borrowed iterator ---
struct MyIterMut<'a, T>(std::slice::IterMut<'a, T>);

impl<'a, T> Iterator for MyIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T> IntoIterator for &'a mut MyVec<T> {
    type Item = &'a mut T;
    type IntoIter = MyIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        MyIterMut(self.0.iter_mut())
    }
}

// Usage
fn iterator_demo() {
    let mut vec = MyVec(vec![1, 2, 3, 4]);

    // Borrowed iterator: map, filter, collect
    let doubled_even: Vec<_> = (&vec).into_iter()
        .map(|&x| x * 2)
        .filter(|&x| x % 4 == 0)
        .collect();
    println!("{:?}", doubled_even); // [4, 8]
    println!("{:?}", vec);

    (&mut vec).into_iter().for_each(|x| *x = *x * 2);
    println!("{:?}", vec);

    // Mutable iterator: modify in place
    for x in (&mut vec).into_iter() {
        *x += 10;
    }
    println!("{:?}", vec);

    // Owned iterator: fold
    let sum: i32 = vec.into_iter().fold(0, |acc, x| acc + x);
    println!("{}", sum); // 1+2+3+4 = 10, plus 40 = 50? Wait careful: after +10, values are 11,12,13,14 sum=50
}

#[test]
fn test_iterator_demo() {
    iterator_demo();
}