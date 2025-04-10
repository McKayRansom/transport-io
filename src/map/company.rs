use serde::{Deserialize, Serialize};

pub type Money = u32;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Company {
    money: Money,
}

pub const STARTING_MONEY: Money = 1000;

#[derive(PartialEq, Eq, Debug)]
pub enum MoneyError {
    InsufficientFunds,
}

impl Company {
    pub fn new() -> Self {
        Self {
            money: STARTING_MONEY,
        }
    }

    pub fn get_money(&self) -> Money {
        return self.money;
    }

    pub fn deposit(&mut self, amount: Money) {
        self.money = self.money.saturating_add(amount)
    }

    pub fn try_withdraw(&mut self, amount: Money) -> Result<(), MoneyError> {
        match self.money.checked_sub(amount) {
            Some(new_balance) => {
                self.money = new_balance;
                Ok(())
            }
            None => Err(MoneyError::InsufficientFunds),
        }
    }
}

#[cfg(test)]
mod company_tests {
    use super::*;

    #[test]
    fn test_withdraw() {
        let mut company = Company::new();
        assert_eq!(company.money, STARTING_MONEY);

        assert_eq!(company.try_withdraw(STARTING_MONEY), Ok(()));
        assert_eq!(company.money, 0);

        assert_eq!(company.try_withdraw(STARTING_MONEY), Err(MoneyError::InsufficientFunds));
        assert_eq!(company.money, 0);
    }
}