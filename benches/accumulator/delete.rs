/// See https://bheisler.github.io/criterion.rs/book/getting_started.html to add more benchmarks.
#[macro_use]
extern crate criterion;

use accumulator::group::{ClassGroup, Rsa2048, UnknownOrderGroup};
use accumulator::hash::hash_to_prime;
use accumulator::{Accumulator, AccumulatorWithHashToPrime, MembershipProof};
use criterion::Criterion;

use rug::Integer;

fn bench_delete<G: UnknownOrderGroup>(
    acc: &Accumulator<G, Integer, AccumulatorWithHashToPrime>,
    c_proof: &MembershipProof<G, Integer, AccumulatorWithHashToPrime>,
) {
    let c = hash_to_prime("c");
    acc.clone()
        .delete_with_proof(&[(c, c_proof.witness.clone())])
        .expect("Valid delete expected.");
}

macro_rules! benchmark_delete {
    ($group_type : ty, $criterion: ident) => {
        let a = hash_to_prime("a");
        let b = hash_to_prime("b");
        let c = hash_to_prime("c");

        let group_type_str = String::from(stringify!($group_type)).to_lowercase();
        let acc_0 =
            Accumulator::<$group_type, Integer, AccumulatorWithHashToPrime>::empty().add(&[a, b]);
        let (acc_1, c_proof) = acc_0.clone().add_with_proof(&[c]);
        $criterion.bench_function(format! {"{}_delete", group_type_str}.as_str(), move |b| {
            b.iter(|| bench_delete(&acc_1.clone(), &c_proof))
        });
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_delete! {Rsa2048, c};
    benchmark_delete! {ClassGroup, c};
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
