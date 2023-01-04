# seahourse_contract_wallet
# Built with Seahorse v0.2.5

from seahorse.prelude import *

declare_id('H66wCfRMbgEZaRURnnJw42B3NvWsJZMH8L93TDSyEdaD')

class Safe(Account):
    bump: u8
    recovery_authority: Pubkey
    owner: Pubkey

@instruction
def init_safe(owner: Signer, safe: Empty[Safe], safe_id: str, recovery_authority: Pubkey):
    safe = safe.init(
        payer= owner,
        seeds= ['safe',owner.key(), safe_id]
    )
    safe.owner = owner.key()
    safe.recovery_authority = recovery_authority

@instruction
def withdraw_funds(to: UncheckedAccount,amt: u64,safe: Safe, authority: Signer):
    assert((safe.owner == authority.key())), "Unauthorized Authority"
    safe.transfer_lamports(to,amt)

@instruction
def recover_safe(new_authority: Pubkey, recover_authority: Signer, safe: Safe):
    assert((safe.recovery_authority == recover_authority.key())), "Unauthorized Recovery Authority"
    safe.owner = new_authority