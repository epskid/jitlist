//! extremely comprehensive testing suite

use super::*;

/// test removing one element
#[test]
fn remove_one() {
    let mut jit_list = JITList::new(vec![1, 2, 3, 4, 5]);
    jit_list.remove(1);

    let expected_result = [1, 3, 4, 5];
    for (i, j) in jit_list.into_iter().zip(expected_result.into_iter()) {
        assert_eq!(i, j);
    }
}

/// test removing two elements
#[test]
fn remove_two() {
    let mut jit_list = JITList::new(vec![1, 2, 3, 4, 5]);
    jit_list.remove(1);
    jit_list.remove(1);

    let expected_result = [1, 4, 5];
    for (i, j) in jit_list.into_iter().zip(expected_result.into_iter()) {
        assert_eq!(i, j);
    }
}

/// unprecedentedly, test removing three elements
#[test]
fn remove_three() {
    let mut jit_list = JITList::new(vec![1, 2, 3, 4, 5]);
    jit_list.remove(1);
    jit_list.remove(1);
    jit_list.remove(2);

    let expected_result = [1, 4];
    for (i, j) in jit_list.into_iter().zip(expected_result.into_iter()) {
        assert_eq!(i, j);
    }
}
