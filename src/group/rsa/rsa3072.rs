//! RSA (3072Rsa3072) group using GMP integers in the `rug` crate.
use super::{ElemFrom, ElemToBytes, Group, UnknownOrderGroup};
use crate::util::{int, TypeRep};
use rug::{rand::MutRandState, Integer};
use rug_binserial::Integer as BinInteger;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rsa3072 {}

/// RSA-3072 modulus. Just some number I generated in Sage, you should probably not trust me...
const RSA3072_MODULUS_DECIMAL: &str = "5396081988321530539039402185833488096572272229021121138822446071725950856185310683589779421666891823918123339042063842708368774168874404100411662652951095750576922793737271343857722294383422566816819730087614591092195992679889558250880490895523250010847174336869419971712471615729866383256810351400199152223064369471308184678793367886480824598292668991804658041163195577380759680095977049158803546503417656127120196220613801271153935726384652621645622622073917353203710775654199965914716502378759700072317940026413373380133829273858326378692026963688454438221669555933696731657747774083127569883065710592408962708754975440812690655581022900122477934694731911544942204586464370757574765678455498530891915334684288289508815722962356263836449735863880393293060492411599007849890700818110089413317199189263525804606530969671768289155072329771862866933433356482308046334996967878764279195155574918324801936131345714525673673789351";

lazy_static! {
  pub static ref RSA3072_MODULUS: Integer = Integer::from_str(RSA3072_MODULUS_DECIMAL).unwrap();
  pub static ref HALF_MODULUS: Integer = RSA3072_MODULUS.clone() / 2;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// An RSA 3072 group element, directly wrapping a GMP integer from the `rug` crate.
pub struct Rsa3072Elem(BinInteger);

impl TypeRep for Rsa3072 {
  type Rep = Integer;
  fn rep() -> &'static Self::Rep {
    &RSA3072_MODULUS
  }
}

impl Group for Rsa3072 {
  type Elem = Rsa3072Elem;
  fn op_(modulus: &Integer, a: &Rsa3072Elem, b: &Rsa3072Elem) -> Rsa3072Elem {
    Self::elem(int(a.0.as_ref() * b.0.as_ref()) % modulus)
  }

  fn id_(_: &Integer) -> Rsa3072Elem {
    Self::elem(1)
  }

  fn inv_(modulus: &Integer, x: &Rsa3072Elem) -> Rsa3072Elem {
    Self::elem(x.0.as_ref().invert_ref(modulus).unwrap())
  }

  fn exp_(modulus: &Integer, x: &Rsa3072Elem, n: &Integer) -> Rsa3072Elem {
    // A side-channel resistant impl is 40% slower; we'll consider it in the future if we need to.
    Self::elem(x.0.as_ref().pow_mod_ref(n, modulus).unwrap())
  }
}

impl<T> ElemFrom<T> for Rsa3072
where
  Integer: From<T>,
{
  fn elem(t: T) -> Rsa3072Elem {
    let modulus = Self::rep();
    let val = int(t) % modulus;
    if val > *HALF_MODULUS {
      Rsa3072Elem(
        <(Integer, Integer)>::from((-val).div_rem_euc_ref(&modulus))
          .1
          .into(),
      )
    } else {
      Rsa3072Elem(val.into())
    }
  }
}

impl ElemToBytes for Rsa3072 {
  fn elem_to_bytes(val: &Rsa3072Elem) -> Vec<u8> {
    let num = val.0.clone();
    let digits = num.as_ref().significant_digits::<u8>();
    let mut bytes = vec![0u8; digits];
    num
      .as_ref()
      .write_digits(&mut bytes, rug::integer::Order::MsfBe);
    bytes
  }
}

impl UnknownOrderGroup for Rsa3072 {
  fn unknown_order_elem_(_: &Integer) -> Rsa3072Elem {
    Self::elem(2)
  }

  fn unknown_possibly_random_order_elem_<R: MutRandState>(
    _: &Self::Rep,
    rng: &mut R,
  ) -> Rsa3072Elem {
    Self::elem(Self::order_upper_bound().random_below(rng))
  }

  fn order_upper_bound_(_: &Integer) -> Integer {
    HALF_MODULUS.clone()
  }

  fn rsa_modulus_(_: &Integer) -> Result<Integer, Integer> {
    Ok(RSA3072_MODULUS.clone())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_init() {
    let _x = &Rsa3072::rep();
  }

  #[test]
  fn test_op() {
    let a = Rsa3072::op(&Rsa3072::elem(2), &Rsa3072::elem(3));
    assert!(a == Rsa3072::elem(6));
    let b = Rsa3072::op(&Rsa3072::elem(-2), &Rsa3072::elem(-3));
    assert!(b == Rsa3072::elem(6));
  }

  /// Tests that `-x` and `x` are treated as the same element.
  #[test]
  fn test_cosets() {
    assert!(Rsa3072::elem(3) == Rsa3072::elem(RSA3072_MODULUS.clone() - 3));
    // TODO: Add a trickier coset test involving `op`.
  }

  #[test]
  fn test_inv() {
    let x = Rsa3072::elem(2);
    let inv = Rsa3072::inv(&x);
    assert!(Rsa3072::op(&x, &inv) == Rsa3072::id());
  }
}
