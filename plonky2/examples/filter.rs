use anyhow::Result;
// use env_logger::init;
use plonky2::field::types::Field;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
// use serde::de::value::BoolDeserializer;

const N: usize = 5;

fn main() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
    let query = builder.add_virtual_target();
    let initial_keys = builder.add_virtual_targets(N);
    let initial_vals = builder.add_virtual_targets(N);
    let initial_matches = initial_keys
        .iter()
        .map(|k| builder.is_equal(*k, query))
        .collect::<Vec<_>>();
    let num_matches = initial_matches
        .iter()
        .fold(builder.zero(), |acc, m| builder.add(acc, m.target));
    // shift n times
    let mut matches = initial_matches
        .into_iter()
        .map(|x| x.target)
        .collect::<Vec<_>>();
    let mut vals = initial_vals.clone();
    let zero = builder.zero();
    for _ in 0..(N - 1) {
        (0..N).fold(builder._false(), |moving, i| {
            let not_match = builder.not(BoolTarget::new_unsafe(matches[i]));
            let moving = builder.or(moving, not_match);
            let next_match = if i == N - 1 { zero } else { matches[i + 1] };
            let next_val = if i == N - 1 { zero } else { vals[i + 1] };
            matches[i] = builder._if(moving, next_match, matches[i]);
            vals[i] = builder._if(moving, next_val, vals[i]);
            moving
        });
    }
    let final_keys = matches
        .iter()
        .map(|m| builder.mul(*m, query))
        .collect::<Vec<_>>();

    // public inputs
    builder.register_public_input(query);
    builder.register_public_inputs(&initial_keys);
    builder.register_public_inputs(&initial_vals);
    let num_public_inputs = 1 + 2 * N;
    // generated public outpus
    builder.register_public_input(num_matches);
    builder.register_public_inputs(&final_keys);
    builder.register_public_inputs(&vals);

    let mut pw = PartialWitness::new();
    // pw.set_target(query, F::from_canonical_usize(N - 1));
    // pw.set_target_arr(
    //     &initial_keys,
    //     &(0..N)
    //         .map(|i| F::from_canonical_usize(i))
    //         .collect::<Vec<_>>(),
    // );
    // pw.set_target_arr(
    //     &initial_vals,
    //     &(0..N)
    //         .map(|i| F::from_canonical_usize(i + N))
    //         .collect::<Vec<_>>(),
    // );
    pw.set_target(query, F::from_canonical_usize(2));
    pw.set_target_arr(
        &initial_keys,
        &[2, 0, 2, 1, 2]
            .iter()
            .map(|i| F::from_canonical_usize(*i))
            .collect::<Vec<_>>(),
    );
    pw.set_target_arr(
        &initial_vals,
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
