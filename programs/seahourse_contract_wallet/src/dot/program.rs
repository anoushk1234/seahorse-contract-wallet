#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{assign, id, index_assign, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct Safe {
    pub bump: u8,
    pub recovery_authority: Pubkey,
    pub owner: Pubkey,
}

impl<'info, 'entrypoint> Safe {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedSafe<'info, 'entrypoint>> {
        let bump = account.bump;
        let recovery_authority = account.recovery_authority.clone();
        let owner = account.owner.clone();

        Mutable::new(LoadedSafe {
            __account__: account,
            __programs__: programs_map,
            bump,
            recovery_authority,
            owner,
        })
    }

    pub fn store(loaded: Mutable<LoadedSafe>) {
        let mut loaded = loaded.borrow_mut();
        let bump = loaded.bump;

        loaded.__account__.bump = bump;

        let recovery_authority = loaded.recovery_authority.clone();

        loaded.__account__.recovery_authority = recovery_authority;

        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;
    }
}

#[derive(Debug)]
pub struct LoadedSafe<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Safe>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub bump: u8,
    pub recovery_authority: Pubkey,
    pub owner: Pubkey,
}

pub fn init_safe_handler<'info>(
    mut owner: SeahorseSigner<'info, '_>,
    mut safe: Empty<Mutable<LoadedSafe<'info, '_>>>,
    mut safe_id: String,
    mut recovery_authority: Pubkey,
) -> () {
    let mut safe = safe.account.clone();

    assign!(safe.borrow_mut().owner, owner.key());

    assign!(safe.borrow_mut().recovery_authority, recovery_authority);
}

pub fn recover_safe_handler<'info>(
    mut new_authority: Pubkey,
    mut recover_authority: SeahorseSigner<'info, '_>,
    mut safe: Mutable<LoadedSafe<'info, '_>>,
) -> () {
    if !(safe.borrow().recovery_authority == recover_authority.key()) {
        panic!("Unauthorized Recovery Authority");
    }

    assign!(safe.borrow_mut().owner, new_authority);
}

pub fn withdraw_funds_handler<'info>(
    mut to: UncheckedAccount<'info>,
    mut amt: u64,
    mut safe: Mutable<LoadedSafe<'info, '_>>,
    mut authority: SeahorseSigner<'info, '_>,
) -> () {
    if !(safe.borrow().owner == authority.key()) {
        panic!("Unauthorized Authority");
    }

    {
        let amount = amt;

        **safe
            .borrow()
            .__account__
            .to_account_info()
            .try_borrow_mut_lamports()
            .unwrap() -= amount;

        **to.to_account_info().try_borrow_mut_lamports().unwrap() += amount;
    };
}
