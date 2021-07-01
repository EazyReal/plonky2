use crate::circuit_builder::CircuitBuilder;
use crate::field::extension_field::target::ExtensionTarget;
use crate::field::extension_field::Extendable;
use crate::target::Target;
use crate::generator::EqualityGenerator;

impl<F: Extendable<D>, const D: usize> CircuitBuilder<F, D> {
    /// Evaluates to 1 if `x` equals zero, 0 otherwise.
    pub fn is_zero(&mut self, x: Target) -> Target {
        let m = todo!();
        let y = todo!();

        self.add_generator(EqualityGenerator {
            x,
            m,
            y,
        });

        y
    }

    /// Evaluates to 1 if `x` and `y` are equal, 0 otherwise.
    pub fn is_equal(&mut self, x: Target, y: Target) -> Target {
        self.is_zero(self.sub(x, y))
    }

    /// Inserts a `Target` in a vector at a non-deterministic index. This is done by rotating to the
    /// left, inserting at 0 and then rotating to the right.
    /// Note: `index` is not range-checked.
    pub fn insert(
        &mut self,
        index: Target,
        element: ExtensionTarget<D>,
        v: Vec<ExtensionTarget<D>>,
    ) -> Vec<ExtensionTarget<D>> {
        let mut already_inserted = self.zero();
        let mut new_list = Vec::new();

        for i in 0..v.len() {
            let one = self.one();
            
            let cur_index = self.constant(F::from_canonical_usize(i));
            let insert_here = self.is_equal(cur_index, index);

            let mut new_item = self.zero_extension();
            new_item = self.scalar_mul_add_extension(insert_here, element, new_item);
            if i > 0 {
                new_item =
                    self.scalar_mul_add_extension(already_inserted, v[i - 1], new_item);
            }
            already_inserted = self.add(already_inserted, insert_here);

            let not_already_inserted = self.sub(one, already_inserted);
            new_item = self.scalar_mul_add_extension(not_already_inserted, v[i], new_item);

            new_list.push(new_item);
        }

        new_list
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit_data::CircuitConfig;
    use crate::field::crandall_field::CrandallField;
    use crate::field::extension_field::quartic::QuarticCrandallField;
    use crate::field::field::Field;
    use crate::witness::PartialWitness;

    fn real_insert<const D: usize>(
        index: usize,
        element: ExtensionTarget<D>,
        v: &[ExtensionTarget<D>],
    ) -> Vec<ExtensionTarget<D>> {
        let mut res = v.to_vec();
        res.insert(index, element);
        res
    }

    fn test_insert_given_len(len_log: usize) {
        type F = CrandallField;
        type FF = QuarticCrandallField;
        let len = 1 << len_log;
        let config = CircuitConfig::large_config();
        let mut builder = CircuitBuilder::<F, 4>::new(config);
        let v = (0..len - 1)
            .map(|_| builder.constant_extension(FF::rand()))
            .collect::<Vec<_>>();

        for i in 0..len {
            let it = builder.constant(F::from_canonical_usize(i));
            let elem = builder.constant_extension(FF::rand());
            let inserted = real_insert(i, elem, &v);
            let purported_inserted = builder.insert(it, elem, v.clone());

            for (x, y) in inserted.into_iter().zip(purported_inserted) {
                builder.route_extension(x, y);
            }
        }

        let data = builder.build();
        let proof = data.prove(PartialWitness::new());
    }

    #[test]
    fn test_insert() {
        for len_log in 1..3 {
            test_insert_given_len(len_log);
        }
    }
}
