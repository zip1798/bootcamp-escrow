use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};

#[derive(Accounts)]
pub struct Exchange<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mut)]
    pub taker: Signer<'info>,

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

    // Account of Taker for token_b
    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_b: InterfaceAccount<'info, TokenAccount>,
        
    // Account of Maker for token_b
    #[account(
        init_if_needed, // чи потрібно ініцювати як мют
        payer = maker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // Account of Taker for token_a
    #[account(
        init_if_needed, // чи потрібно ініцювати як мют
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn exchange(ctx: Context<Exchange>, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
    // 1) CPI Transfer token_a from maker to taker

    // Prepare accounts for CPI (Cross Program Invocation)
    let transfer_accounts_1 = TransferChecked {
        from: ctx.accounts.maker_token_account_a.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
        to: ctx.accounts.taker_token_account_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };

    // Create context for CPI
    let cpi_context_1 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts_1,
    );

    // CPI instruction
    transfer_checked(
        cpi_context_1,
        token_a_amount,
        ctx.accounts.token_mint_a.decimals,
    )?;
    // ===========================================================


    // 2) CPI Transfer token_b from taker to maker

    // Prepare accounts for CPI (Cross Program Invocation)
    let transfer_accounts_2 = TransferChecked {
        from: ctx.accounts.taker_token_account_b.to_account_info(),
        mint: ctx.accounts.token_mint_b.to_account_info(),
        to: ctx.accounts.maker_token_account_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    // Create context for CPI
    let cpi_context_2 = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts_2,
    );

    // CPI instruction
    transfer_checked(
        cpi_context_2,
        token_b_amount,
        ctx.accounts.token_mint_b.decimals,
    )

}