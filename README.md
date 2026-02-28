# jitlist
pronounced "hitlist", like the name "jose". it's the epitome of data structures. optimized for optimization.

## seriously
that bit about being optimized for optimization is true. it's for optimizing a list of bytecode instructions by removal and replacement.
it supports no insertion or iteration. only indexing and removal. it's O(1) removal and -- technically, 'cause it's jit compiled -- O(1)
indexing.

## why?
n/a

## wow! these benchmarks are very comprehensive and not at all cherry picked!
```
against_vec                                  fastest       │ slowest       │ median        │ mean    
├─ big_fellow_remove_first_100_iter_jitlist  929.6 µs      │ 1.521 ms      │ 956.4 µs      │ 1.014 ms
├─ big_fellow_remove_first_100_iter_vec      23.13 ms      │ 28.37 ms      │ 23.73 ms      │ 23.98 ms
├─ big_fellow_remove_first_jitlist           8.095 µs      │ 53.91 µs      │ 10.54 µs      │ 13.89 µs
├─ big_fellow_remove_first_vec               226.6 µs      │ 1.143 ms      │ 235.7 µs      │ 257.1 µs
├─ big_fellow_summer_jitlist                 559.6 ms      │ 849.1 ms      │ 621.1 ms      │ 628.4 ms
╰─ big_fellow_summer_vec                     2.346 s       │ 4.401 s       │ 2.549 s       │ 2.728 s 
```
don't look into it, but jitlist destroys rust's built in vector struct in these fair benchmarks.
