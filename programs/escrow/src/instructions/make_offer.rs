use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};

use crate::{Offer,ANCHOR_DISCRIMINATOR}; // PDA state account

// Accounts of the instruction MakeOffer with rules
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    
    #[account(mint::token_program = token_program)] // програма яка управляє цим аккаунтом є програмою мінта токенів спл
    pub token_mint_a: InterfaceAccount<'info, Mint>,
    
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    // Account of Maker for token_a
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,
    
    // PDA account Offer
    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    // PDA account for save token_a, authority - offer account
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


// Method of the createing instruction send token_a from maker to vault
pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    // Prepare accounts for CPI (Cross Program Invocation)
    let transfer_accounts = TransferChecked {
        from: context.accounts.maker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        to: context.accounts.vault.to_account_info(),
        authority: context.accounts.maker.to_account_info(),
    };

    // Create context for CPI
    let cpi_context = CpiContext::new(
        context.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    // CPI instruction
    transfer_checked(
        cpi_context,
        token_a_offered_amount,
        context.accounts.token_mint_a.decimals,
    )
}

// Set count of token_b for offer (payer - maker)
pub fn save_offer(context: Context<MakeOffer>, id: u64, token_b_wanted_amount: u64) -> Result<()> {
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_b_wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}
