use anchor_lang::prelude::*;

use anchor_spl::{
    token::{
        self,
        Mint,
    }
};
use std::convert::TryFrom;

declare_id!("E7WGku7aoDV9GHh3cagcrraktA4nETXuCearm1WZvMNU");

#[constant]
pub const DECIMALS: u8 = 9;
pub const BLOCK_GEN_RATE: u8 = 3;
pub const MAX_TOTAL_SUPPLY : u64 = 210_000_000 * 10_u64.pow(DECIMALS as u32);
pub const INIT_COIN_NUMS_PER_BLOCK: u64 = 3 * 10_u64.pow(DECIMALS as u32);  // 30_000_000_000
pub const ARENA_PERCENTAGE: f64 = 0.24;
pub const NFT_MINING_PERCENTAGE: f64 = 0.36;
pub const LIQUIDITY_MINING_PERCENTAGE: f64 = 0.22666666666666666666666666666667;
pub const MARKETING_PERCENTAGE: f64 = 0.04;
pub const ECOSYSTEM_PERCENTAGE: f64 = 0.13333333333333333333333333333333;
    
#[program]
pub mod token_faucet {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        nonce: u8
    ) -> ProgramResult {
        
        msg!("program id:{}", ctx.program_id);
        msg!("config acount ower:{}", ctx.accounts.faucet_config_account.to_account_info().owner);

        if ctx.accounts.receiver_arena.data_is_empty() || 
           ctx.accounts.receiver_nft_mining.data_is_empty() || 
           ctx.accounts.receiver_liquidity_mining.data_is_empty() || 
           ctx.accounts.receiver_marketing.data_is_empty() || 
           ctx.accounts.receiver_ecosystem.data_is_empty(){
            return Err(TokenFaucetError::NotInitilizedAssociatedTokenAccount.into());
        }

        let faucet_config_account = &mut ctx.accounts.faucet_config_account;
        faucet_config_account.token_program = *ctx.accounts.token_program.key;
        faucet_config_account.token_mint = *ctx.accounts.token_mint.to_account_info().key;
        faucet_config_account.token_authority = *ctx.accounts.token_authority.key;

        faucet_config_account.receiver_arena = *ctx.accounts.receiver_arena.key;
        faucet_config_account.receiver_nft_mining = *ctx.accounts.receiver_nft_mining.key;
        faucet_config_account.receiver_liquidity_mining = *ctx.accounts.receiver_liquidity_mining.key;
        faucet_config_account.receiver_marketing = *ctx.accounts.receiver_marketing.key;
        faucet_config_account.receiver_ecosystem = *ctx.accounts.receiver_ecosystem.key;
        
        faucet_config_account.magic = 0x544b4654;
        faucet_config_account.nonce = nonce;
        faucet_config_account.current_block_height = 0;
        faucet_config_account.last_gen_block_timestamp = ctx.accounts.clock.unix_timestamp;
        faucet_config_account.coin_nums_per_block = INIT_COIN_NUMS_PER_BLOCK;

        msg!("Token program key: {}", faucet_config_account.token_program);
        msg!("Token mint key: {}", faucet_config_account.token_mint);
        msg!("token authority key: {}", faucet_config_account.token_authority);
        msg!("Receiver arena key: {}", faucet_config_account.receiver_arena);
        msg!("Receiver nft mining key: {}", faucet_config_account.receiver_nft_mining);
        msg!("Receiver liquidity mining key: {}", faucet_config_account.receiver_liquidity_mining);
        msg!("Receiver marketing key: {}", faucet_config_account.receiver_marketing);
        msg!("Receiver ecosystem key: {}", faucet_config_account.receiver_ecosystem);
        msg!("Magic: 0x{:x}", faucet_config_account.magic);
        msg!("Nonce: {}", faucet_config_account.nonce);
        msg!("Current block height: {}", faucet_config_account.current_block_height);
        msg!("Last gen block timestamp: {}", faucet_config_account.last_gen_block_timestamp);
        msg!("Coin nums per block: {}\n", faucet_config_account.coin_nums_per_block);
        
        Ok(())
    }

    pub fn drip(ctx: Context<Drip>) -> ProgramResult {
                
        msg!("program id:{}", ctx.program_id);
        msg!("config acount ower:{}", ctx.accounts.faucet_config_account.to_account_info().owner);

        let current_time = ctx.accounts.clock.unix_timestamp;
        
        msg!("Current Token Supply: {}", ctx.accounts.token_mint.supply);
        if ctx.accounts.token_mint.supply >= MAX_TOTAL_SUPPLY {            
            return Err(TokenFaucetError::TotalSupplyLimit.into());
        }

        let faucet_config_account = &mut ctx.accounts.faucet_config_account;
        if current_time < faucet_config_account.last_gen_block_timestamp {
            return Err(TokenFaucetError::InvalidUnixTimestamp.into());
        }
        
        let intervals = current_time.checked_sub(faucet_config_account.last_gen_block_timestamp).unwrap();
        msg!("Called Interval: {}s", intervals);
        if intervals < i64::try_from(BLOCK_GEN_RATE).unwrap() {
            return Err(TokenFaucetError::InsufficientIntervalError.into());
        }
        
        let remain_seconds_temp = (u64::try_from(intervals).unwrap()).checked_rem(u64::try_from(BLOCK_GEN_RATE).unwrap()).unwrap();
        msg!("Remain seconds: {}", remain_seconds_temp);

        let gen_block_nums = (u64::try_from(intervals).unwrap()).checked_div(u64::try_from(BLOCK_GEN_RATE).unwrap()).unwrap();

        let distribution_amounts = gen_block_nums.checked_mul(faucet_config_account.coin_nums_per_block).unwrap();

        msg!("Current block height: {}", faucet_config_account.current_block_height);
        msg!("Current coin number of per block: {}", faucet_config_account.coin_nums_per_block);
        msg!("This time distribution token total amounts: {}", distribution_amounts);

        let receiver_arena_amount: u64 = ((distribution_amounts as f64) * ARENA_PERCENTAGE) as u64;
        let receiver_nft_mining_amount: u64 = ((distribution_amounts as f64) * NFT_MINING_PERCENTAGE) as u64;
        let receiver_liquidity_mining_amount: u64 = ((distribution_amounts as f64) * LIQUIDITY_MINING_PERCENTAGE) as u64;
        let receiver_marketing_amount: u64 = ((distribution_amounts as f64) * MARKETING_PERCENTAGE) as u64;
        let receiver_ecosystem_amount: u64 = ((distribution_amounts as f64) * ECOSYSTEM_PERCENTAGE) as u64;

        msg!("This time arena should be distributed token's amounts: {}", receiver_arena_amount);
        msg!("This time nft_mining should be distributed token's amounts: {}", receiver_nft_mining_amount);
        msg!("This time liquidity_mining should be distributed token's amounts: {}", receiver_liquidity_mining_amount);
        msg!("This time marketing should be distributed token's amounts: {}", receiver_marketing_amount);
        msg!("This time ecosystem should be distributed token's amounts: {}", receiver_ecosystem_amount);
        
        faucet_config_account.current_block_height = faucet_config_account.current_block_height.checked_add(gen_block_nums).unwrap();
        faucet_config_account.last_gen_block_timestamp = current_time.checked_sub(i64::try_from(remain_seconds_temp).unwrap()).unwrap();
        msg!("Update block height, now block height: {}", faucet_config_account.current_block_height);
        msg!("Update timestamp, now last gen block timestamp: {}", faucet_config_account.last_gen_block_timestamp);

        msg!("Start mint to receivers");
        ctx.accounts.token_mint_to(ctx.accounts.receiver_arena.clone(), receiver_arena_amount)?;
        ctx.accounts.token_mint_to(ctx.accounts.receiver_nft_mining.clone(), receiver_nft_mining_amount)?;
        ctx.accounts.token_mint_to(ctx.accounts.receiver_liquidity_mining.clone(), receiver_liquidity_mining_amount)?;
        ctx.accounts.token_mint_to(ctx.accounts.receiver_marketing.clone(), receiver_marketing_amount)?;
        ctx.accounts.token_mint_to(ctx.accounts.receiver_ecosystem.clone(), receiver_ecosystem_amount)?;
        msg!("Mint to receivers finished");
        msg!("Finished...\n\n");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize <'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"GameYoo-Token".as_ref()],
        bump,
        rent_exempt = enforce
    )]
    pub faucet_config_account: Account<'info, FaucetConfigAccount>,

    #[account()]
    pub user: Signer<'info>,

    #[account(address = token::ID @ TokenFaucetError::InvalidTokenProgram)]
    pub token_program: AccountInfo<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"GYC-Mint-Auth".as_ref()],
        bump = bump
    )]
    pub token_authority: AccountInfo<'info>,

    pub receiver_arena: AccountInfo<'info>,

    pub receiver_nft_mining: AccountInfo<'info>,

    pub receiver_liquidity_mining: AccountInfo<'info>,

    pub receiver_marketing: AccountInfo<'info>,

    pub receiver_ecosystem: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    pub clock: Sysvar<'info, Clock>,

    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct Drip<'info> {

    #[account(
        mut,
        seeds = [b"GameYoo-Token".as_ref()],
        bump,
        has_one = token_mint @TokenFaucetError::InvalidTokenMint,
        has_one = token_authority @TokenFaucetError::InvalidTokenAuthority,
        has_one = receiver_arena @TokenFaucetError::InvalidReceiverTokenAccount,
        has_one = receiver_nft_mining @TokenFaucetError::InvalidReceiverTokenAccount,
        has_one = receiver_liquidity_mining @TokenFaucetError::InvalidReceiverTokenAccount,
        has_one = receiver_marketing @TokenFaucetError::InvalidReceiverTokenAccount,
        has_one = receiver_ecosystem @TokenFaucetError::InvalidReceiverTokenAccount,
        constraint = faucet_config_account.magic == 0x544b4654 @TokenFaucetError::InvalidMagic,
    )]  
    pub faucet_config_account: Account<'info, FaucetConfigAccount>,

    #[account(address = token::ID @ TokenFaucetError::InvalidTokenProgram)]
    pub token_program: AccountInfo<'info>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"GYC-Mint-Auth".as_ref()],
        bump = faucet_config_account.nonce
    )]
    pub token_authority: AccountInfo<'info>,

    #[account(mut)]
    pub receiver_arena: AccountInfo<'info>,

    #[account(mut)]
    pub receiver_nft_mining: AccountInfo<'info>,

    #[account(mut)]
    pub receiver_liquidity_mining: AccountInfo<'info>,

    #[account(mut)]
    pub receiver_marketing: AccountInfo<'info>,

    #[account(mut)]
    pub receiver_ecosystem: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>
}

#[account]
pub struct FaucetConfigAccount {
    pub magic: u32,
    pub nonce: u8,
    pub current_block_height: u64,
    pub last_gen_block_timestamp: i64,
    pub coin_nums_per_block: u64,

    pub token_program: Pubkey,
    pub token_mint: Pubkey,
    pub token_authority: Pubkey,

    pub receiver_arena: Pubkey,
    pub receiver_nft_mining: Pubkey,
    pub receiver_liquidity_mining: Pubkey,
    pub receiver_marketing: Pubkey,
    pub receiver_ecosystem: Pubkey
}

impl Default for FaucetConfigAccount {

    fn default() -> FaucetConfigAccount {
        unsafe { std::mem::zeroed()}
    }

}

impl<'info> Drip<'info> {
    fn token_mint_to(&mut self, receiver: AccountInfo<'info>, amount: u64) -> ProgramResult {
        let cpi_program = self.token_program.clone();
        let cpi_accounts = token::MintTo {
            authority: self.token_authority.to_account_info(),
            mint: self.token_mint.to_account_info(),
            to: receiver.to_account_info()
        };  

        let seeds = &[b"GYC-Mint-Auth".as_ref(), &[self.faucet_config_account.nonce]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }
}

#[error]
pub enum TokenFaucetError {
    #[msg("This is an error message that contain invalid params.")]
    InvalidParamError,
    #[msg("Insufficient interval to generate a block.")]
    InsufficientIntervalError,
    #[msg("Token Supply has reached the max limit.")]
    TotalSupplyLimit,
    #[msg("Config account is invalid.")]
    InvalidConfigAccount,
    #[msg("Invalid magic number.")]
    InvalidMagic,
    #[msg("Invalid config owner.")]
    InvalidConfigOwner,
    #[msg("Invalid token receiver account.")]
    InvalidReceiverTokenAccount,
    #[msg("Not initilized token associated account.")]
    NotInitilizedAssociatedTokenAccount,
    #[msg("Invalid token authority.")]
    InvalidTokenAuthority,
    #[msg("Invalid token mint.")]
    InvalidTokenMint,
    #[msg("Invalid token program.")]
    InvalidTokenProgram,
    #[msg("Invalid unix timestamp.")]
    InvalidUnixTimestamp
}