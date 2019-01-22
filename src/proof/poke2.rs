use super::super::group::{Group, InvertibleGroup};
use super::super::hash::hashes;
use super::super::util;
use num::{BigInt, BigUint};
use num_bigint::Sign::Plus;
use num_integer::Integer;
use serde::ser::Serialize;

#[allow(non_snake_case)]
#[derive(PartialEq)]
pub struct PoKE2<T> {
  z: T,
  Q: T,
  r: BigUint,
}

/// See page 16 of B&B.
pub fn prove_poke2<G: InvertibleGroup>(
  base: &G::Elem,
  exp: &BigInt,
  result: &G::Elem,
) -> PoKE2<G::Elem> {
  let g = G::base_elem();
  let z = G::exp_signed(&g, exp);
  let l = hash_prime(base, result, &z);
  let alpha = hash_inputs(base, result, &z, &l);
  let q = exp.div_floor(&BigInt::from_biguint(Plus, l.clone()));
  let r = util::mod_euc_big(exp, &l);
  PoKE2 {
    z,
    Q: G::exp_signed(&G::op(&base, &G::exp(&g, &alpha)), &q),
    r,
  }
}

/// See page 16 of B&B.
pub fn verify_poke2<G: Group>(base: &G::Elem, result: &G::Elem, proof: &PoKE2<G::Elem>) -> bool {
  #[allow(non_snake_case)]
  let PoKE2 { z, Q, r } = proof;
  let g = G::base_elem();
  let l = hash_prime(base, result, &z);
  let alpha = hash_inputs(base, result, &z, &l);
  let lhs = G::op(
    &G::exp(Q, &l),
    &G::exp(&G::op(&base, &G::exp(&g, &alpha)), &r),
  );
  let rhs = G::op(result, &G::exp(&z, &alpha));
  lhs == rhs
}

fn hash_prime<G: Serialize>(_u: &G, _w: &G, _z: &G) -> BigUint {
  // TODO: Replace with commented out when hash_prime is implemented.
  BigUint::from(13 as u8)
  // let mut hash_string = serde_json::to_string(&u).unwrap();
  // hash_string.push_str(&serde_json::to_string(&w).unwrap());
  // hash_string.push_str(&serde_json::to_string(&z).unwrap());
  // hashes::h_prime(&hashes::blake2, hash_string.as_bytes())
}

fn hash_inputs<G: Serialize>(u: &G, w: &G, z: &G, l: &BigUint) -> BigUint {
  let mut hash_string = serde_json::to_string(&u).unwrap();
  hash_string.push_str(&serde_json::to_string(&w).unwrap());
  hash_string.push_str(&serde_json::to_string(&z).unwrap());
  hash_string.push_str(&l.to_str_radix(16));
  hashes::blake2(hash_string.as_bytes(), None)
}

#[cfg(test)]
mod tests {
  use super::super::super::group::dummy::DummyRSA;
  use super::*;

  #[test]
  fn test_poke2() {
    // 2^20 = 1048576
    let base = DummyRSA::base_elem();
    let exp = BigInt::from(20 as u8);
    let result = DummyRSA::elem_of(1_048_576);
    let proof = prove_poke2::<DummyRSA>(&base, &exp, &result);
    assert!(verify_poke2::<DummyRSA>(&base, &result, &proof));
    // Must compare entire structs since elements z, Q, and r are private
    assert!(
      proof
        == PoKE2 {
          z: DummyRSA::elem_of(1048576),
          Q: DummyRSA::elem_of(130463359518971),
          r: BigUint::from(7 as u8)
        }
    );

    // 2^35 = 34359738368
    let exp_2 = BigInt::from(35 as u8);
    let result_2 = DummyRSA::elem_of(34_359_738_368);
    let proof_2 = prove_poke2::<DummyRSA>(&base, &exp_2, &result_2);
    assert!(verify_poke2::<DummyRSA>(&base, &result_2, &proof_2));
    // Cannot verify wrong base/exp/result triple with wrong pair.
    assert!(!verify_poke2::<DummyRSA>(&base, &result_2, &proof));
    assert!(
      proof_2
        == PoKE2 {
          z: DummyRSA::elem_of(34_359_738_368),
          Q: DummyRSA::elem_of(909_043_872_400_063),
          r: BigUint::from(9 as u8)
        }
    );
  }

  #[test]
  fn test_poke2_negatives() {
    let base = DummyRSA::elem_of(2);
    let exp = BigInt::from((-5) as i8);
    let result = DummyRSA::exp_signed(&base, &exp);
    let proof = prove_poke2::<DummyRSA>(&base, &exp, &result);
    assert!(verify_poke2::<DummyRSA>(&base, &result, &proof));
    assert!(
      proof
        == PoKE2 {
          z: DummyRSA::elem_of(1_135_351_933_874_355),
          Q: DummyRSA::elem_of(586_139_831_188_592),
          r: BigUint::from(8 as u8)
        }
    );
  }

  #[test]
  fn test_hash_inputs() {
    let base = DummyRSA::elem_of(2);
    let exp = BigInt::from(20 as u8);
    let result = DummyRSA::elem_of(1_048_576);
    let z = DummyRSA::exp_signed(&DummyRSA::base_elem(), &exp);
    let l = hash_prime(&base, &result, &z);
    let alpha = hash_inputs(&base, &result, &z, &l);
    assert!(
      alpha
        == BigUint::new(vec![
          3652804667, 2122523887, 324677495, 3968534693, 1956023477, 4290210450, 3126358525,
          845356874
        ])
    );
  }
}
