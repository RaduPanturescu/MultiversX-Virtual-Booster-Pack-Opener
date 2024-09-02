/*
 * imports
 */


#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod nft_module;

use nft_module::Rarity;


/* ************************* */


/*
 *  Contract
 * declaration
 */


#[elrond_wasm::contract]
pub trait NftMinter: nft_module::NftModule
{
    /*
     *   Init
     * function
     */


    #[init]
    fn init(&self)
	{
        /* Red Booster */
        
        self.add_booster(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                         1,
                         6,
                         59500,
                         27000,
                         6500,
                         4500,
                         2500);
        
        self.add_booster_guarantee(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                                   1,
                                   Rarity::Rare,
                                   1);
        
        self.set_booster_constraints(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                                     1,
                                     0,
                                     0,
                                     3,
                                     2,
                                     1);
        

        /* Blue Booster */
        
        self.add_booster(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                         2,
                         6,
                         63000,
                         22500,
                         9500,
                         4000,
                         1000);
        
        self.add_booster_guarantee(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                                   2,
                                   Rarity::Uncommon,
                                   1);
        
        self.set_booster_constraints(TokenIdentifier::from(&b"BONPACKS-f0b549"[..]),
                                     2,
                                     0,
                                     0,
                                     3,
                                     2,
                                     1);
        
	}


    /* *************************************************** */
}


/* ***************************************************** */