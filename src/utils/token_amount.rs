use {alloy::primitives::U256, std::cmp::Ordering};

/// Basically a BigInt utility, but restricted to U256 and u8 types
#[derive(Debug, Clone)]
pub struct TokenAmount {
    amount: U256,
    decimals: u8,
}

impl TokenAmount {
    pub fn new(amount: U256, decimals: u8) -> Self {
        Self { amount, decimals }
    }
}

impl PartialOrd for TokenAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(compare_amounts(
            self.amount,
            other.amount,
            self.decimals,
            other.decimals,
        ))
    }
}

impl PartialEq for TokenAmount {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// Compare two amounts with different decimals
fn compare_amounts(a_amount: U256, b_amount: U256, a_decimals: u8, b_decimals: u8) -> Ordering {
    // Same assertion as alloy's Unit type
    assert!(a_decimals <= 77);
    assert!(b_decimals <= 77);

    let base_10 = U256::from(10);
    match a_decimals.cmp(&b_decimals) {
        Ordering::Equal => a_amount.cmp(&b_amount),
        Ordering::Greater => {
            // A has more decimals than B
            let diff = a_decimals - b_decimals;
            let exp = U256::from(diff as u64);
            let factor = base_10.checked_pow(exp);
            if let Some(factor) = factor {
                let b_amount_adjusted = b_amount.checked_mul(factor);
                if let Some(b_amount_adjusted) = b_amount_adjusted {
                    a_amount.cmp(&b_amount_adjusted)
                } else {
                    Ordering::Less
                }
            } else {
                // Branch should never be reached because 2^256 > 10^77
                Ordering::Less
            }
        }
        Ordering::Less => {
            // A has less decimals than B
            let diff = b_decimals - a_decimals;
            let exp = U256::from(diff as u64);
            let factor = base_10.checked_pow(exp);
            if let Some(factor) = factor {
                let a_amount_adjusted = a_amount.checked_mul(factor);
                if let Some(a_amount_adjusted) = a_amount_adjusted {
                    a_amount_adjusted.cmp(&b_amount)
                } else {
                    Ordering::Greater
                }
            } else {
                // Branch should never be reached because 2^256 > 10^77
                Ordering::Greater
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_amounts_0() {
        let a_amount = TokenAmount {
            amount: U256::from(0),
            decimals: 0,
        };
        let b_amount = TokenAmount {
            amount: U256::from(0),
            decimals: 0,
        };
        assert_eq!(a_amount, b_amount);
        assert_eq!(a_amount.partial_cmp(&b_amount), Some(Ordering::Equal));
        assert!(a_amount <= b_amount);
        assert!(a_amount >= b_amount);
        assert!(b_amount <= a_amount);
        assert!(b_amount >= a_amount);
        assert!(!(a_amount < b_amount));
        assert!(!(a_amount > b_amount));
    }

    #[test]
    fn test_equal_amounts_different_decimals() {
        let a_amount = TokenAmount {
            amount: U256::from(0),
            decimals: 10,
        };
        let b_amount = TokenAmount {
            amount: U256::from(0),
            decimals: 0,
        };
        assert_eq!(a_amount, b_amount);
        assert_eq!(a_amount.partial_cmp(&b_amount), Some(Ordering::Equal));
        assert!(a_amount <= b_amount);
        assert!(a_amount >= b_amount);
        assert!(b_amount <= a_amount);
        assert!(b_amount >= a_amount);
        assert!(!(a_amount < b_amount));
        assert!(!(a_amount > b_amount));
    }

    #[test]
    fn test_different_amounts() {
        let a_amount = TokenAmount {
            amount: U256::from(1),
            decimals: 0,
        };
        let b_amount = TokenAmount {
            amount: U256::from(0),
            decimals: 0,
        };
        assert!(a_amount > b_amount);
        assert!(a_amount >= b_amount);
        assert!(!(a_amount < b_amount));
        assert!(!(a_amount <= b_amount));
        assert!(!(a_amount == b_amount));
        assert!(b_amount < a_amount);
        assert!(b_amount <= a_amount);
        assert!(!(b_amount > a_amount));
        assert!(!(b_amount >= a_amount));
        assert!(!(b_amount == a_amount));
    }

    #[test]
    fn test_different_decimals() {
        let a_amount = TokenAmount {
            amount: U256::from(1),
            decimals: 0,
        };
        let b_amount = TokenAmount {
            amount: U256::from(1),
            decimals: 1,
        };
        assert!(a_amount > b_amount);
        assert!(a_amount >= b_amount);
        assert!(!(a_amount < b_amount));
        assert!(!(a_amount <= b_amount));
        assert!(!(a_amount == b_amount));
        assert!(b_amount < a_amount);
        assert!(b_amount <= a_amount);
        assert!(!(b_amount > a_amount));
        assert!(!(b_amount >= a_amount));
        assert!(!(b_amount == a_amount));
    }

    #[test]
    fn test_overflow_amount() {
        let a_amount_amount = U256::from(1000000);
        let a_amount_decimals = 0;
        let b_amount_amount = U256::from(1);
        let b_amount_decimals = 77;
        assert!(U256::from(10)
            .checked_pow(U256::from(b_amount_decimals - a_amount_decimals))
            .is_some());
        assert!(a_amount_amount
            .checked_mul(U256::from(
                U256::from(10)
                    .checked_pow(U256::from(b_amount_decimals - a_amount_decimals))
                    .unwrap()
            ))
            .is_none());
        let a_amount = TokenAmount {
            amount: a_amount_amount,
            decimals: a_amount_decimals,
        };
        let b_amount = TokenAmount {
            amount: b_amount_amount,
            decimals: b_amount_decimals,
        };
        assert!(a_amount > b_amount);
        assert!(a_amount >= b_amount);
        assert!(!(a_amount < b_amount));
        assert!(!(a_amount <= b_amount));
        assert!(!(a_amount == b_amount));
        assert!(b_amount < a_amount);
        assert!(b_amount <= a_amount);
        assert!(!(b_amount > a_amount));
        assert!(!(b_amount >= a_amount));
        assert!(!(b_amount == a_amount));
    }
}
