use divan::Bencher;

use jitlist::JITList;

fn main() {
    divan::main();
}

fn sorted_gen_jitlist(size: usize) -> impl FnMut() -> JITList<i32> {
    move || JITList::new(sorted_gen_vec(size)())
}

fn sorted_gen_vec(size: usize) -> impl FnMut() -> Vec<i32> {
    move || (0..size).map(|i| i as i32).collect()
}

#[divan::bench]
fn big_fellow_remove_first_jitlist(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_jitlist(1 << 20))
        .bench_local_refs(|r| r.remove(0));
}

#[divan::bench]
fn big_fellow_remove_first_vec(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_vec(1 << 20))
        .bench_local_refs(|r| r.remove(0));
}

#[divan::bench]
fn big_fellow_remove_first_100_iter_jitlist(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_jitlist(1 << 20))
        .bench_local_refs(|r| {
            for i in 0..100 {
                r.remove(i);
            }

            for i in r.into_iter().take(100) {
                std::hint::black_box(i);
            }
        });
}

#[divan::bench]
fn big_fellow_remove_first_100_iter_vec(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_vec(1 << 20))
        .bench_local_refs(|r| {
            for i in 0..100 {
                r.remove(i);
            }

            for i in r.into_iter().take(100) {
                std::hint::black_box(i);
            }
        });
}

#[divan::bench]
fn big_fellow_summer_jitlist(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_jitlist(1 << 20))
        .bench_local_refs(|r| {
            for i in 0..1_000 {
                let mut total = 0;
                for _ in 0..10 {
                    total += r[i + 1];
                    r.remove(i + 1);
                }
                r[i] = total;
            }
        });
}

#[divan::bench]
fn big_fellow_summer_vec(bencher: Bencher) {
    bencher
        .with_inputs(sorted_gen_vec(1 << 20))
        .bench_local_refs(|r| {
            for i in 0..1_000 {
                let mut total = 0;
                for _ in 0..10 {
                    total += r[i + 1];
                    r.remove(i + 1);
                }
                r[i] = total;
            }
        });
}
