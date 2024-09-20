use anchor_lang::{prelude::*, solana_program::system_instruction};
use std::mem::size_of;

declare_id!("6kgSdKsaQGrWMVrCgp7RmXX7pnqVnDZ5JDJjTDvC2j62");

#[program]
pub mod solana_contracts 
{
    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let owner_account = &mut ctx.accounts.owner_account; 
        require! (!owner_account.owner_initialized, ErrorCode::AlreadyInitialized);
        
            owner_account.owner_pubkey = ctx.accounts.caller.key();
            owner_account.owner_initialized = true; 
        
        Ok(())
    }

    pub fn add_early_sale_addresses(ctx: Context<EarlySaleContext>, addresses: Vec<Pubkey>) -> Result<()>
    {
        let owner_account = &ctx.accounts.owner_account;
        require! (owner_account.owner_pubkey == ctx.accounts.caller.key(), ErrorCode::NotAuthorized);    
            for _user in addresses 
            {
                let early_sale_account = &mut ctx.accounts.early_sale_account;
                early_sale_account.in_early_sale = true; 
            }
        Ok(())  
    }

    pub fn buy_node(ctx: Context<BuyNodeContext>, quantity:u64, amount:u64,tier_number: u64) -> Result<()>
    {
        let buy_node_account = &mut ctx.accounts.buy_node_account;
        let tier_account = &mut ctx.accounts.tier_account;
        let funds_handler_account = &ctx.accounts.funds_handler_account;
        let nodes_bought_account = &mut ctx.accounts.nodes_bought_account;

        require!(quantity <= tier_account.tier_limit[tier_number as usize],ErrorCode::QuantityOutOfBounds);
        require!(tier_number < 12 && tier_number > 0,ErrorCode::TierLimit);   
            if buy_node_account.early_sale_on
            {
                require!(buy_node_account.in_early_sale,ErrorCode::EarlySale);
                require!(amount == ( quantity * tier_account.tier_price[tier_number as usize]),ErrorCode::IncorrectAmount);
                let ix = system_instruction::transfer
                (   &ctx.accounts.caller.key(), 
                    &funds_handler_account.funds_handler.key(),
                    amount 
                );

                anchor_lang::solana_program::program::invoke
                (   &ix, 
                    &[
                        ctx.accounts.caller.to_account_info(),
                        funds_handler_account.to_account_info(),        
                     ],
                )?;

                nodes_bought_account.nodes_bought += 1;
                tier_account.tier_limit[tier_number as usize] -= 1;         
            }
            else
            {
                require!(amount == ( quantity * tier_account.tier_price[tier_number as usize]),ErrorCode::IncorrectAmount);
                require!(tier_account.tier_limit[tier_number as usize] > 0,ErrorCode::TierLimit);
                let ix = system_instruction::transfer
                (   &ctx.accounts.caller.key(), 
                    &funds_handler_account.funds_handler.key(),
                    amount 
                );

                anchor_lang::solana_program::program::invoke
                (   &ix, 
                    &[
                        ctx.accounts.caller.to_account_info(),
                        funds_handler_account.to_account_info(),        
                     ],
                )?;

                nodes_bought_account.nodes_bought += 1;
                tier_account.tier_limit[tier_number as usize] -= 1;         
            }
        emit!(NodeBought{
            caller: *ctx.accounts.caller.key,
            quantity: quantity,
            amount: amount,
            tier_number: tier_number,
            total_nodes_bought: nodes_bought_account.nodes_bought,
            pending_tier_limit: tier_account.tier_limit[tier_number as usize] 
        });
        Ok(())
    }

    /// @dev Setter functions
    pub fn set_funds_handler(ctx: Context<FundsHandlerContext>,new_funds_handler: Pubkey) -> Result<()>
    {
        let owner_account = &ctx.accounts.owner_account; 
        require!(owner_account.owner_pubkey == ctx.accounts.caller.key(),ErrorCode::NotAuthorized);
        let funds_handler_account = &mut ctx.accounts.funds_handler_account;
        funds_handler_account.funds_handler = new_funds_handler; 
        emit!(NewFundsHandler{
            funds_handler: new_funds_handler
        });
        Ok(())
    }

    pub fn set_early_sale_on(ctx: Context<BuyNodeContext>, sale_type: bool) -> Result<()>
    {
        let owner_account = &ctx.accounts.owner_account; 
        require!(owner_account.owner_pubkey == ctx.accounts.caller.key(),ErrorCode::NotAuthorized);
        let buy_node_account = &mut ctx.accounts.buy_node_account;
        buy_node_account.early_sale_on = sale_type; 
        emit!(EarlySale{
            early_sale: sale_type
        });
        Ok(())
    }

    pub fn set_tier_limit(ctx: Context<TierContext>,new_tier_limit: u64,tier_number: u64) -> Result<()>
    {
        let owner_account = &ctx.accounts.owner_account; 
        require!(owner_account.owner_pubkey == ctx.accounts.caller.key(),ErrorCode::NotAuthorized);
        let tier_account = &mut ctx.accounts.tier_account;
        require!(tier_number < 12 && tier_number > 0,ErrorCode::TierLimit);
        tier_account.tier_limit[tier_number as usize] = new_tier_limit;
        emit!(TierLimit{
            tier_limit: new_tier_limit,
            tier_number: tier_number
        });
        Ok(())
    }

    pub fn set_tier_price(ctx: Context<TierContext>,new_price:u64,tier_number: u64) -> Result<()>
    {
        let owner_account = &ctx.accounts.owner_account; 
        require!(owner_account.owner_pubkey == ctx.accounts.caller.key(),ErrorCode::NotAuthorized);
        let tier_account = &mut ctx.accounts.tier_account; 
        tier_account.tier_price[tier_number as usize] = new_price;
        emit!(TierPrice{
            tier_price: new_price,
            tier_number: tier_number
        });
        Ok(())
    }

    /// @dev Getter functions 
    pub fn get_funds_handler(ctx: Context<FundsHandlerContext>) -> Result<Pubkey>
    {
        let funds_handler_account = &ctx.accounts.funds_handler_account;
        Ok(funds_handler_account.funds_handler)
    }

    pub fn get_tier_limit(ctx: Context<TierContext>,tier_number: u64) -> Result<u64>
    {
        let tier_account = &ctx.accounts.tier_account;
        Ok(tier_account.tier_limit[tier_number as usize])
    }

    pub fn get_early_sale_on(ctx: Context<BuyNodeContext>) -> Result<bool>
    {
        let buy_node_account = &ctx.accounts.buy_node_account; 
        Ok(buy_node_account.early_sale_on)
    }

    pub fn get_tier_price(ctx: Context<TierContext>,tier_number: u64) -> Result<u64>
    {
        let tier_account = &ctx.accounts.tier_account;
        Ok(tier_account.tier_price[tier_number as usize])
    }

    pub fn get_owner(ctx: Context<BuyNodeContext>) -> Result<Pubkey> {
        let owner_account = &ctx.accounts.owner_account;
        Ok(owner_account.owner_pubkey)
    }

    pub fn get_total_nodes_owned(ctx: Context<GetNodeContext>) -> Result<u64> {
        let nodes_bought_account = &ctx.accounts.nodes_bought_account;
        Ok(nodes_bought_account.nodes_bought)
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> 
{
    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<Owner>() + 16
    )]
    pub owner_account: Account<'info, Owner>,  

    #[account(mut)]
    pub caller: Signer<'info>, 
    pub system_program: Program<'info, System>, 
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct EarlySaleContext<'info>
{
    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<Owner>() + 16
    )]
    pub owner_account: Account<'info, Owner>,  

    #[account(
        init,
        payer = caller,
        seeds = [user.key().as_ref()], 
        bump,
        space = size_of::<BuyNode>() + 16
    )]
    pub early_sale_account: Account<'info, BuyNode>, 

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info,System>,
}

#[derive(Accounts)] 
pub struct BuyNodeContext<'info> 
{
    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<Owner>() + 16
    )]
    pub owner_account: Account<'info, Owner>, 

    #[account(
        init, 
        payer = caller, 
        seeds = [b"buy_node_account"], 
        bump,
        space = size_of::<BuyNode>() + 16
    )]
    pub buy_node_account: Account<'info,BuyNode>,

    #[account(
        init, 
        payer = caller, 
        seeds = [b"tier_account"], 
        bump,space = size_of::<Tier>() + 16
    )]
    pub tier_account: Account<'info,Tier>,

    #[account(
        init, 
        payer = caller, 
        seeds = [b"funds_handler_account"], 
        bump,space = size_of::<Tier>() + 16
    )]
    pub funds_handler_account: Account<'info,FundsHandler>,

    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key().to_string().as_bytes()], 
        bump,
        space = size_of::<BuyNode>() + 16
    )]
    pub nodes_bought_account: Account<'info, BuyNode>, 

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info,System>,
}

#[derive(Accounts)]
pub struct FundsHandlerContext<'info>
{
    #[account(
        init, 
        payer = caller, 
        seeds = [b"funds_handler_account"], 
        bump,space = size_of::<Tier>() + 16
    )]
    pub funds_handler_account: Account<'info,FundsHandler>,

    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<Owner>() + 16
    )]
    pub owner_account: Account<'info, Owner>,  

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info,System>,
}

#[derive(Accounts)]
pub struct TierContext<'info>
{
    #[account(
        init, 
        payer = caller, 
        seeds = [b"tier_account"], 
        bump,space = size_of::<Tier>() + 16
    )]
    pub tier_account: Account<'info,Tier>,

    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<Owner>() + 16
    )]
    pub owner_account: Account<'info, Owner>,  

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info,System>,
}

#[derive(Accounts)]
pub struct GetNodeContext<'info>
{
    #[account(
        init, 
        payer = caller, 
        seeds = [caller.key.as_ref()], 
        bump,
        space = size_of::<BuyNode>() + 16
    )]
    pub nodes_bought_account: Account<'info, BuyNode>,

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info,System>,
}

#[account]
pub struct Owner 
{
    pub owner_pubkey: Pubkey,
    pub owner_initialized:bool
}

#[account]
pub struct BuyNode
{
    pub early_sale_on: bool,
    pub total_nodes: u64,
    pub in_early_sale: bool,
    pub nodes_bought: u64
}

#[account]
pub struct Tier
{
    pub tier_limit: Vec<u64>,
    pub tier_price: Vec<u64>
} 

#[account]
pub struct FundsHandler
{
    pub funds_handler: Pubkey,
}

#[event]
pub struct Initialize
{
    pub caller: Pubkey,
    pub quantity: u64, 
    pub amount: u64
}

#[event]
pub struct NodeBought
{
    pub caller: Pubkey,
    pub quantity: u64, 
    pub amount: u64,
    pub tier_number: u64,
    pub total_nodes_bought: u64,
    pub pending_tier_limit: u64
}

#[event]
pub struct NewFundsHandler
{
    pub funds_handler: Pubkey,
}

#[event]
pub struct EarlySale
{
    pub early_sale: bool,
}

#[event]
pub struct TierLimit
{
    pub tier_limit: u64,
    pub tier_number: u64
}

#[event]
pub struct TierPrice
{
    pub tier_price:u64,
    pub tier_number: u64
}

#[error_code]
pub enum ErrorCode 
{
    #[msg("Alreay initialized!")]
    AlreadyInitialized,

    #[msg("Not Authorized!")]
    NotAuthorized,

    #[msg("Out of tier limits!")]
    TierLimit, 

    #[msg("Quantity is more than the available nodes in the tier!")]
    QuantityOutOfBounds, 

    #[msg("Not part of early sale!")]
    EarlySale, 

    #[msg("Incorrect Amount!")]
    IncorrectAmount,
}