use anyhow::Result;
// use itertools::izip;
use log::LevelFilter;
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
// use serde::de::value::BoolDeserializer;

const N: usize = 5;
const N_LOG: usize = 7;
const N_BIT: usize = 16;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
    let query = builder.add_virtual_target();
    let keys = builder.add_virtual_targets(N);
    let vals = builder.add_virtual_targets(N);
    let matches = keys
        .iter()
        .map(|k| builder.is_equal(*k, query).target)
        .collect::<Vec<_>>();
    // let zero_target = builder.zero();

    // get num matches
    let num_matches = matches
        .iter()
        .fold(builder.zero(), |acc, m| builder.add(acc, *m));

    // encode the tuples
    let shift_match_target = builder.constant(F::from_canonical_u64(1 << (N_BIT + N_LOG)));
    let shift_index_target = builder.constant(F::from_canonical_u64(1 << (N_BIT)));
    let compact = matches
        .iter()
        .zip(vals.iter())
        .enumerate()
        .map(|(i, (m, v))| {
            let shifted_match = builder.mul(*m, shift_match_target);
            let index = builder.constant(F::from_canonical_usize(N - i));
            let shifted_index_add_val = builder.mul_add(index, shift_index_target, *v);
            builder.add(shifted_match, shifted_index_add_val)
        })
        .collect::<Vec<_>>();

    // sort the encoded inputs
    let sorted_compact: Vec<Target> =
        builder.sort::<<C as GenericConfig<D>>::Hasher>(&compact, 1 + N_BIT + N_LOG);

    // decode the inputs
    let sorted_matches = sorted_compact
        .iter()
        .map(|c| {
            let bits = builder.split_le(*c, 1 + N_BIT + N_LOG);
            bits[N_BIT + N_LOG].target
        })
        .collect::<Vec<_>>();
    let sorted_keys = sorted_matches
        .iter()
        .map(|m| builder.mul(*m, query))
        .collect::<Vec<_>>();
    let sorted_vals = sorted_compact
        .iter()
        .zip(sorted_matches.iter())
        .map(|(c, m)| {
            // this is a convenient way to get the low bits
            // we can also do bit composition only and call `builder.le_sum` since we only need to get low bits
            let (low, _) = builder.split_low_high(*c, N_BIT, 64);
            builder.mul(low, *m)
        })
        .collect::<Vec<_>>();

    // public inputs
    builder.register_public_input(query);
    builder.register_public_inputs(&keys);
    builder.register_public_inputs(&vals);
    let num_public_inputs = 1 + 2 * N;
    // generated public outpus
    builder.register_public_input(num_matches);
    builder.register_public_inputs(&sorted_keys);
    builder.register_public_inputs(&sorted_vals);

    // print stats
    builder.print_gate_counts(0);

    let mut pw = PartialWitness::new();
    // pw.set_target(query, F::from_canonical_usize(N - 1));
    // pw.set_target_arr(
    //     &keys,
    //     &(0..N)
    //         .map(|i| F::from_canonical_usize(i))
    //         .collect::<Vec<_>>(),
    // );
    // pw.set_target_arr(
    //     &vals,
    //     &(0..N)
    //         .map(|i| F::from_canonical_usize(i + N))
    //         .collect::<Vec<_>>(),
    // );
    pw.set_target(query, F::from_canonical_usize(2));
    pw.set_target_arr(
        &keys,
        &[2, 0, 2, 1, 2]
            .iter()
            .map(|i| F::from_canonical_usize(*i))
            .collect::<Vec<_>>(),
    );
    pw.set_target_arr(
        &vals,
        &(0..N)
            .map(|i| F::from_canonical_usize(i + N))
            .collect::<Vec<_>>(),
    );

    let data = builder.build::<C>();
    let proof = data.prove(pw)?;

    println!(
        "number of matches is {}",
        proof.public_inputs[num_public_inputs]
    );
    (0..N).for_each(|i| {
        println!(
            "{} {}",
            proof.public_inputs[num_public_inputs + 1 + i],
            proof.public_inputs[num_public_inputs + N + 1 + i]
        )
    });

    data.verify(proof)
}
