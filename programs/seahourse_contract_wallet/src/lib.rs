#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("H66wCfRMbgEZaRURnnJw42B3NvWsJZMH8L93TDSyEdaD");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }
}

#[program]
mod seahourse_contract_wallet {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (safe_id : String , recovery_authority : Pubkey)]
    pub struct InitSafe<'info> {
        #[account(mut)]
        pub owner: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Safe > () + 8 , payer = owner , seeds = ["safe" . as_bytes () . as_ref () , owner . key () . as_ref () , safe_id . as_bytes () . as_ref ()] , bump)]
        pub safe: Box<Account<'info, dot::program::Safe>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init_safe(
        ctx: Context<InitSafe>,
        safe_id: String,
        recovery_authority: Pubkey,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let safe = Empty {
            account: dot::program::Safe::load(&mut ctx.accounts.safe, &programs_map),
            bump: ctx.bumps.get("safe").map(|bump| *bump),
        };

        init_safe_handler(owner.clone(), safe.clone(), safe_id, recovery_authority);

        dot::program::Safe::store(safe.account);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (new_authority : Pubkey)]
    pub struct RecoverSafe<'info> {
        #[account(mut)]
        pub recover_authority: Signer<'info>,
        #[account(mut)]
        pub safe: Box<Account<'info, dot::program::Safe>>,
    }

    pub fn recover_safe(ctx: Context<RecoverSafe>, new_authority: Pubkey) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let recover_authority = SeahorseSigner {
            account: &ctx.accounts.recover_authority,
            programs: &programs_map,
        };

        let safe = dot::program::Safe::load(&mut ctx.accounts.safe, &programs_map);

        recover_safe_handler(new_authority, recover_authority.clone(), safe.clone());

        dot::program::Safe::store(safe);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (amt : u64)]
    pub struct WithdrawFunds<'info> {
        #[account(mut)]
        #[doc = "CHECK: This account is unchecked."]
        pub to: UncheckedAccount<'info>,
        #[account(mut)]
        pub safe: Box<Account<'info, dot::program::Safe>>,
        #[account(mut)]
        pub authority: Signer<'info>,
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amt: u64) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let to = &ctx.accounts.to.clone();
        let safe = dot::program::Safe::load(&mut ctx.accounts.safe, &programs_map);
        let authority = SeahorseSigner {
            account: &ctx.accounts.authority,
            programs: &programs_map,
        };

        withdraw_funds_handler(to.clone(), amt, safe.clone(), authority.clone());

        dot::program::Safe::store(safe);

        return Ok(());
    }
}
