use crate::field::extension::Extendable;
use crate::hash::hash_types::RichField;
use crate::iop::challenger::RecursiveChallenger;
use crate::iop::target::Target;
use crate::plonk::circuit_builder::CircuitBuilder;
use crate::plonk::config::AlgebraicHasher;

impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilder<F, D> {
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
}
