use vstd::prelude::*;
verus! {

    spec fn is_prime_nat(n : nat) -> bool {
        &&& forall |i : nat| 1 < i < n ==> (#[trigger] (n % i)) != 0
        &&& n != 0
        &&& n != 1
    }

    proof fn bv_to_nat_modulo(n : u64, i : u64)
            requires i < n && i != 0
            ensures (n % i) as nat == (n as nat) % (i as nat)
            {}

    fn is_prime_impl(n : u64)  -> (r : bool)
        ensures r == is_prime_nat(n as nat)
    {
        if (n <= 1) {
            return false;
        }
        let mut i = 2;
        while i < n
            invariant
                2 <= i <= n
                && forall |j:nat| 1 < j < i ==> (#[trigger] (n as nat % j)) != 0
            decreases
                n - i
        {
            if n % i == 0 {
                assert((n as nat) % (i as nat) == 0);
                return false;
            }
            i = i + 1;
        }
        assert(is_prime_nat(n as nat));
        return true;
    }

    spec fn is_square_root(n : nat, x : nat) -> bool {
        forall |j : nat| (#[trigger] (j * j)) <= n ==> j <= x
    }

    proof fn square_root_is_enough(n : nat, x : nat)
        requires
            is_square_root(n, x) &&
            forall |i : nat| 1 < i < x ==> (#[trigger] (x % i)) != 0
        ensures
            is_prime_nat(n)
            {

            }



    fn main() {
    }
}
