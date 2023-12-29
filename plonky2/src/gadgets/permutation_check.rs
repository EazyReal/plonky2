use crate::field::extension::Extendable;
use crate::hash::hash_types::RichField;
use crate::iop::challenger::RecursiveChallenger;
use crate::iop::generator::{GeneratedValues, SimpleGenerator};
use crate::iop::target::Target;
use crate::iop::witness::{PartitionWitness, Witness, WitnessWrite};
use crate::plonk::circuit_builder::CircuitBuilder;
use crate::plonk::circuit_data::CommonCircuitData;
use crate::plonk::config::AlgebraicHasher;
use crate::util::serialization::{Buffer, IoResult, Read, Write};

impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilder<F, D> {
    // sort an array a, generator required
    pub fn sort<H: AlgebraicHasher<F>>(
        &mut self,
        original_array: &[Target],
        n_log: usize,
    ) -> Vec<Target> {
        let sorted_array = self.add_virtual_targets(original_array.len());
        self.sorted_check::<H>(&sorted_array, original_array, n_log);
        self.add_simple_generator(SortGenerator {
            original_array: original_array.to_vec(),
            sorted_array: sorted_array.clone(),
        });
        sorted_array
    }

    // verify that a < b when both are under n_log bits
    pub fn assert_leq(&mut self, smaller: Target, larger: Target, n_log: usize) {
        // (2^16 - 1) - 0 < 2^16
        let diff = self.sub(larger, smaller);
        self.range_check(diff, n_log)
    }

    // verify that a is a sorted version of b
    pub fn sorted_check<H: AlgebraicHasher<F>>(
        &mut self,
        sorted_array: &[Target],
        original_array: &[Target],
        n_log: usize,
    ) {
        // check: a is sorted
        for i in 1..sorted_array.len() {
            self.assert_leq(sorted_array[i], sorted_array[i - 1], n_log);
        }
        // check: a is a permutation of b
        self.permutation_check::<H>(sorted_array, original_array);
    }

    // verify that a is a permutation of b
    pub fn permutation_check<H: AlgebraicHasher<F>>(&mut self, a: &[Target], b: &[Target]) {
        let mut challenger = RecursiveChallenger::<F, H, D>::new(self);
        challenger.observe_elements(a);
        challenger.observe_elements(b);
        let alpha = challenger.get_challenge(self);
        let a_eval = a.iter().fold(self.one(), |acc, x| {
            let term = self.sub(*x, alpha);
            self.mul(acc, term)
        });
        let b_eval = b.iter().fold(self.one(), |acc, x| {
            let term = self.sub(*x, alpha);
            self.mul(acc, term)
        });
        self.connect(a_eval, b_eval);
    }
}

#[derive(Debug, Default)]
pub struct SortGenerator {
    original_array: Vec<Target>,
    sorted_array: Vec<Target>,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for SortGenerator {
    fn id(&self) -> String {
        "SortGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<Target> {
        self.original_array.clone()
    }

    fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
        let mut array = witness
            .get_targets(&self.original_array)
            .into_iter()
            .map(|x| x.to_canonical_u64() as usize)
            .collect::<Vec<_>>();
        array.sort();
        array.reverse();
        out_buffer.set_target_arr(
            &self.sorted_array,
            &array
                .into_iter()
                .map(|x| F::from_canonical_usize(x))
                .collect::<Vec<_>>(),
        );
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        dst.write_target_vec(&self.original_array)?;
        dst.write_target_vec(&self.sorted_array)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        let original_array = src.read_target_vec()?;
        let sorted_array = src.read_target_vec()?;
        Ok(Self {
            original_array,
            sorted_array,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use log::LevelFilter;

    use crate::field::types::Field;
    use crate::gadgets::permutation_check::*;
    use crate::iop::witness::{PartialWitness, WitnessWrite};
    use crate::plonk::circuit_builder::CircuitBuilder;
    use crate::plonk::circuit_data::CircuitConfig;
    use crate::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use crate::plonk::verifier::verify;

    #[test]
    fn test_permutation_check() -> Result<()> {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .init();

        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        // type FF = <C as GenericConfig<D>>::FE;
        const N: usize = 100;
        const K: usize = 3;

        let config = CircuitConfig::standard_recursion_config();

        let mut pw = PartialWitness::<F>::new();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let a = builder.add_virtual_targets(N);
        let b = builder.add_virtual_targets(N);

        builder.permutation_check::<<C as GenericConfig<D>>::Hasher>(&a, &b);

        let a_vals = (0..N)
            .map(|i| F::from_canonical_usize(i))
            .collect::<Vec<_>>();
        let b_vals = (0..N)
            .map(|i| F::from_canonical_usize((i + K) % N))
            .collect::<Vec<_>>();

        println!("a_vals: {:?}", a_vals);
        println!("b_vals: {:?}", b_vals);

        pw.set_target_arr(&a, &a_vals);
        pw.set_target_arr(&b, &b_vals);

        // print stats
        builder.print_gate_counts(0);

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        verify(proof, &data.verifier_only, &data.common)
    }

    #[test]
    fn test_sort() -> Result<()> {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .init();

        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        // type FF = <C as GenericConfig<D>>::FE;
        const N: usize = 100;
        const K: usize = 3;

        let config = CircuitConfig::standard_recursion_config();

        let mut pw = PartialWitness::<F>::new();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let a = builder.add_virtual_targets(N);
        let b = builder.sort::<<C as GenericConfig<D>>::Hasher>(&a, 16);

        // builder.register_public_inputs(&a);
        builder.register_public_inputs(&b);

        let a_vals = (0..N)
            .map(|i| F::from_canonical_usize((i + K) % N))
            .collect::<Vec<_>>();

        println!("a_vals: {:?}", a_vals);

        pw.set_target_arr(&a, &a_vals);

        // print stats
        builder.print_gate_counts(0);

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        println!("sorted_a: {:?}", proof.public_inputs);

        verify(proof, &data.verifier_only, &data.common)
    }
}
