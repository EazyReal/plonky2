# Filtering from an array

## Notations
We view a tuple in the array as a key value pair, stored in two target vectors `(initial_)keys` and `(initial_)vals`, and call the `match` input `query` instead in the code base.

---

## Solution 1: Shifting the array

### Idea and Code Description
- `plonky2/examples/filter.rs` contains the executable
- Use the `is_eq()` gadget to check equality between the `keys` and the `query` target, producing a `matches` target vector.
- We shift the elements of `matches` (instead of `keys` to avoid checking `is_eq` everytime) and `vals` beginning from the first unmatched indice for n-1 times, this will produce the desired filtered tuple (but the firsts of the tuples will be 1 instead of `query`)
  - For each shift, we check if there is any unmatched instance to the left or at the current position by a `moving` intermediate value. And each of the value of i-th position is updated with the value of (i+1)-th position if moving is true.
- Finally, we multiply back the `query` to the matches to ensure the result is in the desired format

### Performance

#### Complexity
- Suppose the number of elements is $n$. The circuit has $O(n^2)$ gates.

#### Empirical Experiment

|               | n=100 | n =200 |
| ------------- | ----- | ------ |
| degree        | 4025  | 16045  |
| padded degree | 4096  | 16384  |

- $n = 100$
```
[DEBUG plonky2::util::context_tree] 3972 gates to root
[DEBUG plonky2::plonk::circuit_builder] Total gate counts:
[DEBUG plonky2::plonk::circuit_builder] - 3972 instances of ArithmeticGate { num_ops: 20 }
[DEBUG plonky2::plonk::circuit_builder] Degree before blinding & padding: 4025
[DEBUG plonky2::plonk::circuit_builder] Degree after blinding & padding: 4096
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 5.6251507s
```


- $n = 200$
```
[DEBUG plonky2::util::context_tree] 15942 gates to root
[DEBUG plonky2::plonk::circuit_builder] Total gate counts:
[DEBUG plonky2::plonk::circuit_builder] - 15942 instances of ArithmeticGate { num_ops: 20 }
[DEBUG plonky2::plonk::circuit_builder] Degree before blinding & padding: 16045
[DEBUG plonky2::plonk::circuit_builder] Degree after blinding & padding: 16384
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 22.66263s
```

--- 

## Solution 2: Sorting with Permutation Check

### Idea
- We can encode the tuple to a new array where the $i$-th index is 
  - is_match|N-index|original_val
- We sort the array and decode to get the desired filtered output
- Sort
  - To "verify" sort, we can just verify the output array is sorted, and the output array is a permutation of the original array.
- Permutation Check
  - To "verify" A is a permutation of B, notice that this is equivalent to $\prod_i (x - A_i) = \prod_i (x - B_i)$ as polynomials.
  - To verify two polynomials are the same, we can apply the Schwartz-Zippel lemma and apply Fiat-Shamir heuristic so we can use the hash of A, B (ordered) as the random number.

### Code Decsription
- Code structure
  - `plonky2/examples/filter_perm.rs` contains the executable
  - `plonky2/src/gadgets/permutation_check.rs` contains the utility functions (e.g. `sort`, `permutation_check`)
- The `filter_bench.rs` part contains the encoding and decoding of the tuples 
- `sort(original_array)`
  - we defined and use a `SortGenerator` struct implementing the `SimpleGenerator` trait to generate the sorted array
  - we verify the generated array (`sorted_array`) is a sorted version of the original array (`original_array`) by
    - checking `assert_leq` of adjacent elements to ensure the array is decreasing (design-wise we can also take a predicate function to make the code more generic)
    - checking `sorted_array` is a permutation of `original_array`
- `assert_leq(a, b, n_log)`
  - Since $b < 2^{\mathrm{n\_log}}$, we can range check $b-a$ to ensure $a \leq b$.
- `permutation_check(a, b)`
  - the idea is described above, we use `RecursiveChallenger` to obtain the challenge

### Performance

#### Complexity
- The complexity is $O(n)$

#### Empirical Experiment

|               | n=100 | n =200 |
| ------------- | ----- | ------ |
| degree        | 613   | 1223   |
| padded degree | 1024  | 2048   |

- $n = 100$
```
[DEBUG plonky2::util::context_tree] 509 gates to root
[DEBUG plonky2::plonk::circuit_builder] Total gate counts:
[DEBUG plonky2::plonk::circuit_builder] - 85 instances of ArithmeticGate { num_ops: 20 }
[DEBUG plonky2::plonk::circuit_builder] - 25 instances of PoseidonGate(PhantomData<plonky2_field::goldilocks_field::GoldilocksField>)<WIDTH=12>
[DEBUG plonky2::plonk::circuit_builder] - 399 instances of BaseSumGate { num_limbs: 63 } + Base: 2
[DEBUG plonky2::plonk::circuit_builder] Degree before blinding & padding: 613
[DEBUG plonky2::plonk::circuit_builder] Degree after blinding & padding: 1024
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 1.2153122s
```

- $n = 200$
```
[DEBUG plonky2::util::context_tree] 1019 gates to root
[DEBUG plonky2::plonk::circuit_builder] Total gate counts:
[DEBUG plonky2::plonk::circuit_builder] - 170 instances of ArithmeticGate { num_ops: 20 }
[DEBUG plonky2::plonk::circuit_builder] - 50 instances of PoseidonGate(PhantomData<plonky2_field::goldilocks_field::GoldilocksField>)<WIDTH=12>
[DEBUG plonky2::plonk::circuit_builder] - 799 instances of BaseSumGate { num_limbs: 63 } + Base: 2
[DEBUG plonky2::plonk::circuit_builder] Degree before blinding & padding: 1223
[DEBUG plonky2::plonk::circuit_builder] Degree after blinding & padding: 2048
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 2.6360862s
```