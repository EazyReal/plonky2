# Filtering from an array

## Notations
We view a tuple in the array as a key value pair, stored in two target vectors `(initial_)keys` and `(initial_)vals`, and call the `match` input `query` instead in the code base.

## Circuit
- Use the `is_eq()` gadget to check equality between the `keys` and the `query` target, producing a `matches` target vector.
- We shift the elements of `matches` (instead of `keys` to avoid checking `is_eq` everytime) and `vals` beginning from the first unmatched indice for n-1 times, this will produce the desired filtered tuple (but the firsts of the tuples will be 1 instead of `query`)
  - For each shift, we check if there is any unmatched instance to the left or at the current position by a `moving` intermediate value. And each of the value of i-th position is updated with the value of (i+1)-th position if moving is true.
- Finally, we multiply back the `query` to the matches to ensure the result is in the desired format

## Potential Improvement
- Suppose the number of elements is $n$. The circuit has $O(n^2)$ gates. A $O(n \log^2 n)$ solution is possible with a sorting network while when $n=100$ the constant may even dominates. 