# Filtering from an array

## Notations
We view a tuple in the array as a key value pair, stored in two target vectors `(initial_)keys` and `(initial_)vals`, and call the `match` input `query` instead in the code base.

## Circuit
- Use the `is_eq()` gadget to check equality between the `keys` and the `query` target, producing a `matches` target vector.
- We shift the elements of `matches` (instead of `keys` to avoid checking `is_eq` everytime) and `vals` beginning from the first unmatched indice for n-1 times, this will produce the desired filtered tuple (but the firsts of the tuples will be 1 instead of `query`)
  - For each shift, we check if there is any unmatched instance to the left or at the current position by a `moving` intermediate value. And each of the value of i-th position is updated with the value of (i+1)-th position if moving is true.
- Finally, we multiply back the `query` to the matches to ensure the result is in the desired format

## Performance
- Suppose the number of elements is $n$. The circuit has $O(n^2)$ gates.
- Empirically,
```
[DEBUG plonky2::util::context_tree] 3972 gates to root
[DEBUG plonky2::plonk::circuit_builder] Total gate counts:
[DEBUG plonky2::plonk::circuit_builder] - 3972 instances of ArithmeticGate { num_ops: 20 }
[DEBUG plonky2::plonk::circuit_builder] Degree before blinding & padding: 4025
[DEBUG plonky2::plonk::circuit_builder] Degree after blinding & padding: 4096
[DEBUG plonky2::plonk::circuit_builder] Building circuit took 5.6251507s
```

## Potential Improvement
- A $O(n \log^2 n)$ solution is possible with a sorting network; however
  - with the bitonic sorting network, the constant may even dominates when $n=100$
  - also, we need a stable result, there are two solutions but both may degrade performance even more
    - stable sorting network
    - add index to the sorted value (originally only 0 and 1)
- Another idea: encode and check sorted.
  - We can encode new_val =  (is_match|99-index|original_val) for each index to sort.
  - Instead of using a sorting network, we can do a permutation check and verify the supplied witness vector is in decreasing order.
  - The hardest part is the permutation check. A potential way to do it (for array A, B) is with proving product_i (x-A_i) = product_i (x-B_i) with Schwartz–Zippel lemma and Fiat-Shamir (poseidon hash is relatively cheap in Plonky2). I think some other method may be available by supplying some information (as a witness) to control a circuit to generate a permutation.