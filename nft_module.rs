/*
 * imports
 */


 elrond_wasm::imports!();
 elrond_wasm::derive_imports!();
 
 
/* ************************** */


/*
 * Struct and
 *    enum
 * declaration
 */


/* NFT struct */
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct NFT<M: ManagedTypeApi>
{
    pub id     : TokenIdentifier<M>,
    pub nonce  : u64,
}

/* Probability struct */
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Probability
{
    pub common_prob    : usize,
    pub uncommon_prob  : usize,
    pub rare_prob      : usize,
    pub epic_prob      : usize,
    pub legendary_prob : usize,
}

/* Constraints struct */
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Constraints
{
    pub common_constr    : u64,
    pub uncommon_constr  : u64,
    pub rare_constr      : u64,
    pub epic_constr      : u64,
    pub legendary_constr : u64,
}

/* Status enum */
#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy, Debug)]
pub enum Status
{
    Frozen,
    Public,
}

/* U64Key enum */
#[derive(TopEncode, TopDecode, NestedEncode, TypeAbi, PartialEq, Clone, Copy, Debug)]
pub enum U64Key
{
    NbrBoostersOpened
}

/* Rarity enum */
#[derive(TopEncode, TopDecode, NestedEncode, TypeAbi, PartialEq, Clone, Copy, Debug)]
pub enum Rarity
{
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/* We implement the
     Rarity enum    */
impl Rarity
{
    /********************************************
     * ~from_u64~                               *
     * ----------                               *
     * Used by the SC to create Rarity from u64 *
     * @param  : (u64) value : value to convert *
     * @return : (Rarity)    : converted value  *
     *******************************************/
    fn from_u64(value: u64) -> Rarity
    {
        /*   We match the
           value to convert */
        match value
        {
            /* Case 0 */
            0 => Rarity::Common,

            /* Case 1 */
            1 => Rarity::Uncommon,

            /* Case 2 */
            2 => Rarity::Rare,

            /* Case 3 */
            3 => Rarity::Epic,

            /* Case 4 */
            4 => Rarity::Legendary,

            /* Default */
            _ =>
                /* This case should never
                   happens due to ~play~
                     function updates     */
                panic!("Unknown rarity value: {}", value),
        }
    }
}


/* ******************************************************************************* */


/*
 *   Module
 * declaration
 */


#[elrond_wasm::module]
pub trait NftModule
{
    /*
     *  Endpoints
     * declaration
     */


    /********************************************
     * ~refill~                                 *
     * --------                                 *
     * Used by the owner to refill the SC cards *
     * by linking them to a specific booster    *
     * @return : (SCResult<()>) SC result       *
     ********************************************/
    #[only_owner]
    #[payable("*")]
    #[endpoint(refill)]
    fn refill(&self,
              #[payment_token] payment_token  : TokenIdentifier,
              #[payment_nonce] payment_nonce  : u64,
              #[payment_amount] payment_amount: BigUint,
                                booster_id    : TokenIdentifier,
                                booster_nonce : u64,
                                rarity        : Rarity
              ) -> SCResult<()>
    {
        /* Issued token shouldn't be EGLD
            and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");

        /*  If the SC does not
           own any of this card */
        if self.blockchain().get_sc_balance(&payment_token, payment_nonce) <= payment_amount
        {
            /* We keep a naive trace of
               this new nft, to be able
               to claim it back without
               alot of computing and gaz */
            self.all_nft().push_back(NFT{id : payment_token.clone(), nonce : payment_nonce});
        }

        /* We declare a variable to determine
                if the SC know this card      */
        let mut card_known: bool = false;

        /* For each Common cards
            in such a Booster   */
        for e in self.cards(&booster_id, booster_nonce, rarity).iter()
        {
            /* If this card is the same
                  as the one issued     */
            if e.id == payment_token && e.nonce == payment_nonce
            {
                /*    We update the
                   variable card_known */
                card_known = true;

                /* We break
                   the loop */
                break;
            }
        }

        /* If the SC does not
             know this card   */
        if !card_known
        {
            /* We add the card to the
            booster's rarity set  */
            self.cards(&booster_id, booster_nonce, rarity).insert(NFT{id : payment_token.clone(), nonce : payment_nonce.clone()});
        }
 
        /* We return
              Ok     */
        Ok(())
    }

    /**************************************************
     * ~burn_booster~                                 *
     * --------------                                 *
     * Used by the user to burn a Booster in order to *
     * open it and claim the cards contained inside   *
     * @return : (SCResult<()>) SC result             *
     *************************************************/
    #[payable("*")]
    #[endpoint(burnBooster)]
    fn burn_booster(&self,
        #[payment_token] payment_token  : TokenIdentifier,
        #[payment_nonce] payment_nonce  : u64,
        #[payment_amount] payment_amount: BigUint,
    ) -> SCResult<()>
    {
        /* The SC should be public */
        require!(self.status().get() == Status::Public, "Invalid Contract status: Contract is frozen");

        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !payment_token.is_egld()
                 && self.boosters_id().contains(&payment_token)
                 && self.boosters_nonce(&payment_token).contains(&payment_nonce), "Invalid token issued: Token is EGLD or unknown Booster");
        
        /*   Only one Booster can
           be open at the same time */
        require!(payment_amount == BigUint::from(1u64), "Invalid Booster quantity: Only one Booster can be open at once");

        /* We get the caller address */
        let caller = self.blockchain().get_caller();

        /* We get the probabilities
              of such a booster     */
        let booster_probabilities = self.boosters_probabilities(&payment_token, payment_nonce).get();

        /* We get the constraints
             of such a booster    */
        let mut booster_constraints = self.boosters_constraints(&payment_token, payment_nonce).get();

        /*  We update the constraints to
           interpret a 0 as no constraints */
        if booster_constraints.common_constr    == 0 {booster_constraints.common_constr    -= 1;}
        if booster_constraints.uncommon_constr  == 0 {booster_constraints.uncommon_constr  -= 1;}
        if booster_constraints.rare_constr      == 0 {booster_constraints.rare_constr      -= 1;}
        if booster_constraints.epic_constr      == 0 {booster_constraints.epic_constr      -= 1;}
        if booster_constraints.legendary_constr == 0 {booster_constraints.legendary_constr -= 1;}

        /* We declare a swap variable */
        let mut rarity_or_quantity: u8 = 0;

        /*    We declare rarity
           and quantity variables */
        let mut rarity: Rarity = Rarity::Common;
        let mut _quantity: u64 = 0;

        /* We declare random variable */
        let mut rand_source = RandomnessSource::<Self::Api>::new();

        /*   For each guarantees
           rarities and quantities */
        for e in self.boosters_guarantees(&payment_token, payment_nonce).iter()
        {
            /* If the value is a rarity */
            if rarity_or_quantity == 0
            {
                /* We set the rarity */
                rarity = Rarity::from_u64(e.get_value_cloned());
            }

            /* If the value is a quantity */
            else
            {
                /* We set the quantity */
                _quantity = e.get_value_cloned();

                /* For each cards
                      to send     */
                for _i in 0.._quantity
                {
                    /* We get the number of different cards  */
                    let nbr_different_cards: usize = self.cards(&payment_token, payment_nonce, rarity).len();

                    /* ~nbr_different_cards~
                       must be greater than 0 */
                    require!(nbr_different_cards > 0, "Invalid cards quantity: No cards is available for such a Booster and rarity");

                    /* We randomly select
                        the card to send  */
                    let rand_res = rand_source.next_usize_in_range(0, nbr_different_cards);

                    /* We get the NFT to send */
                    let nft_to_send = self.cards(&payment_token, payment_nonce, rarity).iter().nth(rand_res).unwrap();

                    /* We send the NFT */
                    self.send().direct(&caller, &nft_to_send.id, nft_to_send.nonce, &BigUint::from(1u64), &[]);
                }
            }

            /* We increment the
                swap variable   */
            rarity_or_quantity = (rarity_or_quantity+1)%2;
        }

        /* We get the number of
            cards left to send  */
        let number_cards_left = self.boosters_cards_quantity(&payment_token, payment_nonce).get() - self.boosters_guarantees_quantity(&payment_token, payment_nonce).get();

        /*   For each guarantees
           rarities and quantities */
        for _i in 0..number_cards_left
        {
            /*   We pick a random number
               between 0 and 100000, which
               represents the probability  */
            let rand_res = rand_source.next_usize_in_range(0, 100001);

            /* We create a variable
               for the result rarity */
            let mut rarity_res = Rarity::Common;

            /* We set this variable the
               correct rarity according
                 to the random result   */
            if rand_res >= booster_probabilities.common_prob && booster_constraints.uncommon_constr > 0
            {
                rarity_res = Rarity::Uncommon
            }
            if rand_res >= booster_probabilities.common_prob + booster_probabilities.uncommon_prob && booster_constraints.rare_constr > 0
            {
                rarity_res = Rarity::Rare
            }
            if rand_res >= booster_probabilities.common_prob + booster_probabilities.uncommon_prob + booster_probabilities.rare_prob
                                                            && booster_constraints.epic_constr > 0
            {
                rarity_res = Rarity::Epic
            }
            if rand_res >= booster_probabilities.common_prob + booster_probabilities.uncommon_prob + booster_probabilities.rare_prob
                                                             + booster_probabilities.epic_prob    && booster_constraints.legendary_constr > 0
            {
                rarity_res = Rarity::Legendary
            }

            /* We get the number of different cards  */
            let nbr_different_cards: usize = self.cards(&payment_token, payment_nonce, rarity_res).len();

            /* ~nbr_different_cards~
               must be greater than 0 */
            require!(nbr_different_cards > 0, "Invalid cards quantity: No cards is available for such a Booster and rarity");

            /* We randomly select
                the card to send  */
            let rand_res = rand_source.next_usize_in_range(0, nbr_different_cards);

            /* We get the NFT to send */
            let nft_to_send = self.cards(&payment_token, payment_nonce, rarity_res).iter().nth(rand_res).unwrap();

            /* We send the NFT */
            self.send().direct(&caller, &nft_to_send.id, nft_to_send.nonce, &BigUint::from(1u64), &[]);

            /*   We match the
               rarity_res value */
            match rarity_res
            {
                /* Case Common */
                Rarity::Common => booster_constraints.common_constr -= 1,

                /* Case Uncommon */
                Rarity::Uncommon => booster_constraints.uncommon_constr -= 1,

                /* Case Rare */
                Rarity::Rare => booster_constraints.rare_constr -= 1,

                /* Case Epic */
                Rarity::Epic => booster_constraints.epic_constr -= 1,

                /* Case Legendary */
                Rarity::Legendary => booster_constraints.legendary_constr -= 1,
            }
        }

        /* We update the number
           of Boosters openned */
        self.u64_datas(U64Key::NbrBoostersOpened).update(|nbr_boosters_opened| *nbr_boosters_opened += 1);

        /* We return
              Ok     */
        return Ok(());
    }

    /*******************************************
     * ~claim_left~                            *
     * ------------                            *
     * Used by the owner to claim the NFT back *
     *******************************************/
    #[only_owner]
    #[endpoint(claimLeft)]
    fn claim_left(&self)
    {
        /* We get the owner address */
        let caller = self.blockchain().get_caller();
 
        /* While the ~all_nft~
            list is not empty  */
         while !self.all_nft().is_empty()
        {
            /* We get the front NFT */
            let nft = self.all_nft().pop_front().unwrap().get_value_cloned();
 
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&nft.id, nft.nonce);
 
            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                     NFT back   */
                self.send().direct(&caller, &nft.id, nft.nonce, &nft_amount, b"claim NFT left");
            }
        }
    }
 
    /******************************************************************************
     * ~claim_booster~                                                            *
     * ---------------                                                            *
     * Used by the owner to claim a specific Booster's NFT back                   *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to claim NFT    *
     * @param  : (u64)             booster_nonce : the booster nonce to claim NFT *
     ******************************************************************************/
    #[only_owner]
    #[endpoint(claimBooster)]
    fn claim_booster(&self, booster_id: TokenIdentifier, booster_nonce: u64)
    {
        /* Issued token shouldn't be EGLD
            and must be a known booster  */
        require!(   !booster_id.is_egld()
                && self.boosters_id().contains(&booster_id)
                && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");

        /* We get the owner address */
        let caller = self.blockchain().get_caller();

        /* For each Common cards
            in such a Booster   */
        for e in self.cards(&booster_id, booster_nonce, Rarity::Common).iter()
        {
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&e.id, e.nonce);

            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                    NFT back   */
                self.send().direct(&caller, &e.id, e.nonce, &nft_amount, b"claim NFT left");
            }
        }

        /* For each Uncommon cards
            in such a Booster    */
        for e in self.cards(&booster_id, booster_nonce, Rarity::Uncommon).iter()
        {
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&e.id, e.nonce);

            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                    NFT back   */
                self.send().direct(&caller, &e.id, e.nonce, &nft_amount, b"claim NFT left");
            }
        }

        /* For each Rare cards
            in such a Booster  */
        for e in self.cards(&booster_id, booster_nonce, Rarity::Rare).iter()
        {
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&e.id, e.nonce);

            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                    NFT back   */
                self.send().direct(&caller, &e.id, e.nonce, &nft_amount, b"claim NFT left");
            }
        }

        /* For each Epic cards
            in such a Booster  */
        for e in self.cards(&booster_id, booster_nonce, Rarity::Epic).iter()
        {
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&e.id, e.nonce);

            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                    NFT back   */
                self.send().direct(&caller, &e.id, e.nonce, &nft_amount, b"claim NFT left");
            }
        }

        /* For each Legendary cards
            in such a Booster     */
        for e in self.cards(&booster_id, booster_nonce, Rarity::Legendary).iter()
        {
            /* We get its amount */
            let nft_amount = self.blockchain().get_sc_balance(&e.id, e.nonce);

            /* If the NFT amount
               is greater than 0 */
            if nft_amount > BigUint::zero()
            {
                /* We claim the
                    NFT back   */
                self.send().direct(&caller, &e.id, e.nonce, &nft_amount, b"claim NFT left");
            }
        }

        /* We remove all Common cards
            for such a Booster     */
        self.cards(&booster_id, booster_nonce, Rarity::Common).clear();

        /* We remove all Uncommon cards
                for such a Booster      */
        self.cards(&booster_id, booster_nonce, Rarity::Uncommon).clear();

        /* We remove all Rare cards
            for such a Booster    */
        self.cards(&booster_id, booster_nonce, Rarity::Rare).clear();

        /* We remove all Epic cards
            for such a Booster    */
        self.cards(&booster_id, booster_nonce, Rarity::Epic).clear();

        /* We remove all Legendary cards
                for such a Booster       */
        self.cards(&booster_id, booster_nonce, Rarity::Legendary).clear();
    }

    /**************************************************************************
     * ~claim_nft~                                                            *
     * -----------                                                            *
     * Used by the owner to claim a specific NFT back                         *
     * @param  : (TokenIdentifier) nft_id    : the booster ID to claim NFT    *
     * @param  : (u64)             nft_nonce : the booster nonce to claim NFT *
     **************************************************************************/
    #[only_owner]
    #[endpoint(claimNFT)]
    fn claim_nft(&self, nft_id: TokenIdentifier, nft_nonce: u64)
    {
        /* We get the owner address */
        let caller = self.blockchain().get_caller();

        /* We get the nft amount */
        let nft_amount = self.blockchain().get_sc_balance(&nft_id, nft_nonce);

        /* If the NFT amount
           is greater than 0 */
        if nft_amount > BigUint::zero()
        {
            /* We claim the
                NFT back   */
            self.send().direct(&caller, &nft_id, nft_nonce, &nft_amount, b"claim NFT left");
        }
    }


    /* ******************************************************************************************************************************************** */


    /*
     *   Storage 
     *  endpoints
     * declaration
     */
    

    /*********************************************
     * ~set_status~                              *
     * ------------                              *
     * Used by the owner to change the SC status *
     * @param  : (Status) status : status to set *
     *********************************************/
    #[only_owner]
    #[endpoint(setStatus)]
    fn set_status(&self, status: Status)
    {
        /* We modify the SC status */
        self.status().set(status);
    }

    /*******************************************************
     * ~set_u64_data~                                      *
     * --------------                                      *
     * Used by the owner to change a U64Data               *
     * @param  : (U64Key) key   : the key of the new value *
     *           (u64)    value : the new value to set     *
     *******************************************************/
    #[only_owner]
    #[endpoint(setU64Data)]
    fn set_u64_data(&self, key: U64Key, value: u64)
    {
        /* We modify the U64Data */
        self.u64_datas(key).set(value);
    }

    /***********************************************************************************************************************
     * ~add_booster~                                                                                                       *
     * -------------                                                                                                       *
     * Used by the owner to add a new type of booster and its probabilities                                                *
     * @param  : (TokenIdentifier) booster_id             : the booster ID to add                                          *
     * @param  : (u64)             booster_nonce          : the booster nonce to add                                       *
     * @param  : (u64)             booster_cards_quantity : the booster cards quantity to add                              *
     * @param  : (usize)           common_prob            : the booster probability to get a Common card for each cards    *
     * @param  : (usize)           uncommon_prob          : the booster probability to get a Uncommon card for each cards  *
     * @param  : (usize)           rare_prob              : the booster probability to get a Rare card for each cards      *
     * @param  : (usize)           epic_prob              : the booster probability to get an Epic card for each cards     *
     * @param  : (usize)           legendary_prob         : the booster probability to get a Legendary card for each cards *
     ***********************************************************************************************************************/
    #[only_owner]
    #[endpoint(addBooster)]
    fn add_booster(&self,
                   booster_id            : TokenIdentifier,
                   booster_nonce         : u64,
                   booster_cards_quantity: u64,
                   common_prob           : usize,
                   uncommon_prob         : usize,
                   rare_prob             : usize,
                   epic_prob             : usize,
                   legendary_prob        : usize)
    {
        /* Probabilities sum must equals 100% (=100000) */
        require!(common_prob + uncommon_prob + rare_prob + epic_prob + legendary_prob == 100000, "Invalid probabilities: probabilities sum must equars 100% (=100000");

        /* We insert the new booster ID */
        self.boosters_id().insert(booster_id.clone());

        /* We insert the new booster nonce */
        self.boosters_nonce(&booster_id).insert(booster_nonce);

        /* We set the new booster card quantity */
        self.boosters_cards_quantity(&booster_id, booster_nonce).set(booster_cards_quantity);

        /* We set the new booster probabilities */
        self.boosters_probabilities(&booster_id, booster_nonce).set(Probability{common_prob    : common_prob,
                                                                                uncommon_prob  : uncommon_prob,
                                                                                rare_prob      : rare_prob,
                                                                                epic_prob      : epic_prob,
                                                                                legendary_prob : legendary_prob});

        /* We set the nex booster default constraints */
        self.boosters_constraints(&booster_id, booster_nonce).set(Constraints{common_constr    : 0u64,
                                                                              uncommon_constr  : 0u64,
                                                                              rare_constr      : 0u64,
                                                                              epic_constr      : 0u64,
                                                                              legendary_constr : 0u64});
    }

    /****************************************************************************************************
     * ~set_booster_cards_quantity~                                                                     *
     * ----------------------------                                                                     *
     * Used by the owner to set the number of cards inside such a booster                               *
     * @param  : (TokenIdentifier) booster_id             : the booster ID to set the cards quantity    *
     * @param  : (u64)             booster_nonce          : the booster nonce to set the cards quantity *
     * @param  : (u64)             booster_cards_quantity : the booster cards quantity to set           *
     ****************************************************************************************************/
    #[only_owner]
    #[endpoint(setBoosterCardsQuantity)]
    fn set_booster_cards_quantity(&self, booster_id: TokenIdentifier, booster_nonce : u64, booster_cards_quantity : u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");

        /* We set the booster cards quantity */
        self.boosters_cards_quantity(&booster_id, booster_nonce).set(booster_cards_quantity);
    }

    /***************************************************************************************************************
     * ~set_booster_probabilities~                                                                                 *
     * ---------------------------                                                                                 *
     * Used by the owner to add guarantees SFT for such a booster                                                  *
     * @param  : (TokenIdentifier) booster_id     : the booster ID to set the probabilities                        *
     * @param  : (u64)             booster_nonce  : the booster nonce to set the probabilities                     *
     * @param  : (usize)           common_prob    : the booster probability to get a Common card for each cards    *
     * @param  : (usize)           uncommon_prob  : the booster probability to get a Uncommon card for each cards  *
     * @param  : (usize)           rare_prob      : the booster probability to get a Rare card for each cards      *
     * @param  : (usize)           epic_prob      : the booster probability to get an Epic card for each cards     *
     * @param  : (usize)           legendary_prob : the booster probability to get a Legendary card for each cards *
     ***************************************************************************************************************/
    #[only_owner]
    #[endpoint(setBoosterProbabilities)]
    fn set_booster_probabilities(&self,
                                 booster_id    : TokenIdentifier,
                                 booster_nonce : u64,
                                 common_prob   : usize,
                                 uncommon_prob : usize,
                                 rare_prob     : usize,
                                 epic_prob     : usize,
                                 legendary_prob: usize)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");

        /* Probabilities sum must equals 100% (=100000) */
        require!(common_prob + uncommon_prob + rare_prob + epic_prob + legendary_prob == 100000, "Invalid probabilities: probabilities sum must equars 100% (=100000");
        
        /* We set the new booster probabilities */
        self.boosters_probabilities(&booster_id, booster_nonce).set(Probability{common_prob    : common_prob,
                                                                                uncommon_prob  : uncommon_prob,
                                                                                rare_prob      : rare_prob,
                                                                                epic_prob      : epic_prob,
                                                                                legendary_prob : legendary_prob});
    }

    /**************************************************************************************
     * ~add_booster_guarantee~                                                            *
     * -----------------------                                                            *
     * Used by the owner to add guarantees SFT for such a booster                         *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to add the guarantee    *
     * @param  : (u64)             booster_nonce : the booster nonce to add the guarantee *
     * @param  : (Rarity)          rarity        : the guaranteed rarity                  *
     * @param  : (u64)             quantity      : the guaranteed quantity                *
     **************************************************************************************/
    #[only_owner]
    #[endpoint(addBoosterGuarantee)]
    fn add_booster_guarantee(&self, booster_id: TokenIdentifier, booster_nonce: u64, rarity: Rarity, quantity: u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");
        
        /* Quantity must be
            greater than 0  */
        require!(quantity > 0, "Invalid guarantees quantity: Guarantees quantity must me greater than 0");

        /* Guarantees cards number
           must not excee Booster
               cards quantity      */
        require!(   self.boosters_guarantees_quantity(&booster_id, booster_nonce).get() + quantity
                 <=
                    self.boosters_cards_quantity(&booster_id, booster_nonce).get(),
                 "Invalid guarantees quantity: Guarantees cards number is greater than Booster cards quantity");
        
        /*  We insert the
           guarantee rarity */
        self.boosters_guarantees(&booster_id, booster_nonce).push_back(rarity as u64);

        /*   We insert the
           guarantee quantity */
        self.boosters_guarantees(&booster_id, booster_nonce).push_back(quantity);

        /*  We increment the
           guarantee quantity */
        self.boosters_guarantees_quantity(&booster_id, booster_nonce).update(|boosters_guarantees_quantity| *boosters_guarantees_quantity += quantity);
    }

    /**************************************************************************************
     * ~set_booster_constraints~                                                          *
     * -------------------------                                                          *
     * Used by the owner to add constraints for such a booster                            *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to add the guarantee    *
     * @param  : (u64)             booster_nonce : the booster nonce to add the guarantee *
     **************************************************************************************/
    #[only_owner]
    #[endpoint(setBoosterConstraints)]
    fn set_booster_constraints(&self,
                               booster_id      : TokenIdentifier,
                               booster_nonce   : u64,
                               common_constr   : u64,
                               uncommon_constr : u64,
                               rare_constr     : u64,
                               epic_constr     : u64,
                               legendary_constr: u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");
        
        /*  We get the number of
           cards in such a Booster */
        let nbr_cards = self.boosters_cards_quantity(&booster_id, booster_nonce).get();

        /*    Constraints must not exceed
           the number of cards in a Booster */
        require!(   common_constr < nbr_cards
                 && uncommon_constr < nbr_cards
                 && rare_constr < nbr_cards
                 && epic_constr < nbr_cards
                 && legendary_constr < nbr_cards,
                 "Invalid constraints values: constraints must not exceed the number of cards in a Booster.");

        self.boosters_constraints(&booster_id, booster_nonce).set(Constraints{common_constr    : common_constr,
                                                                              uncommon_constr  : uncommon_constr,
                                                                              rare_constr      : rare_constr,
                                                                              epic_constr      : epic_constr,
                                                                              legendary_constr : legendary_constr});
    }

    /***************************************************************************
     * ~remove_booster~                                                        *
     * ----------------                                                        *
     * Used by the owner to remove a type of booster and its probabilities     *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to remove    *
     * @param  : (u64)             booster_nonce : the booster nonce to remove *
     ***************************************************************************/
    #[only_owner]
    #[endpoint(removeBooster)]
    fn remove_booster(&self, booster_id: TokenIdentifier, booster_nonce: u64)
    {
        /* We remove the booster nonce */
        self.boosters_nonce(&booster_id).swap_remove(&booster_nonce);

        /* We clear the booster cards quantity */
        self.boosters_cards_quantity(&booster_id, booster_nonce).clear();

        /* We clear the booster probabilities */
        self.boosters_probabilities(&booster_id, booster_nonce).clear();

        /* We clear the booster guarantees */
        self.boosters_guarantees(&booster_id, booster_nonce).clear();

        /* We clear the booster guarantees quantity*/
        self.boosters_guarantees_quantity(&booster_id, booster_nonce).clear();

        /* We clear the booster constraints */
        self.boosters_constraints(&booster_id, booster_nonce).clear();
    }

    /*********************************************************************************************
     * ~clear_booster_probabilities~                                                             *
     * -----------------------------                                                             *
     * Used by the owner to clear a booster's guarantees                                         *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to remove the probabilities    *
     * @param  : (u64)             booster_nonce : the booster nonce to remove the probabilities *
     *********************************************************************************************/
    #[only_owner]
    #[endpoint(clearBoosterProbabilities)]
    fn clear_booster_probabilities(&self, booster_id: TokenIdentifier, booster_nonce: u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");
        
        /* We remove the probabilities */
        self.boosters_probabilities(&booster_id, booster_nonce).clear();
    }

    /******************************************************************************************
     * ~clear_booster_guarantee~                                                              *
     * -------------------------                                                              *
     * Used by the owner to clear a booster's guarantees                                      *
     * @param  : (TokenIdentifier) booster_id    : the booster ID to remove the guarantees    *
     * @param  : (u64)             booster_nonce : the booster nonce to remove the guarantees *
     ******************************************************************************************/
    #[only_owner]
    #[endpoint(clearBoosterGuarantees)]
    fn clear_booster_guarantees(&self, booster_id: TokenIdentifier, booster_nonce: u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");
        
        /*  We clear the guarantees */
        self.boosters_guarantees(&booster_id, booster_nonce).clear();

        /*  We clear the booster guarantees quantity*/
        self.boosters_guarantees_quantity(&booster_id, booster_nonce).clear();
    }

    #[only_owner]
    #[endpoint(clearBoosterConstraints)]
    fn clear_booster_constraints(&self, booster_id: TokenIdentifier, booster_nonce: u64)
    {
        /* Issued token shouldn't be EGLD
             and must be a known booster  */
        require!(   !booster_id.is_egld()
                 && self.boosters_id().contains(&booster_id)
                 && self.boosters_nonce(&booster_id).contains(&booster_nonce), "Invalid token issued: Token is EGLD or unknown Booster");

        self.boosters_constraints(&booster_id, booster_nonce).clear();
    }

    /****************************************************************************************
     * ~remove_card~                                                                        *
     * -------------                                                                        *
     * Used by the owner and the SC to remove a card                                        *
     * @param : (TokenIdentifier) booster_id    : the booster ID to remove the card from    *
     * @param : (u64)             booster_nonce : the booster nonce to remove the card from *
     * @param : (Rarity)          rarity        : the card's rarity                         *
     * @param : (TokenIdentifier) card_id       : the card's ID                             *
     * @param : (u64)             card_nonce    : the card's nonce                          *
     * @param : (nool)            claim         : should the SC send you back the SFT ?     *
     ****************************************************************************************/
    #[only_owner]
    #[endpoint(removeCard)]
    fn remove_card(&self, booster_id: TokenIdentifier, booster_nonce: u64, rarity: Rarity, card_id: TokenIdentifier, card_nonce: u64, claim: bool)
    {
        /* We get the caller address */
        let caller = self.blockchain().get_caller();

        /* If claim is true */
        if claim
        {
            /* We claim theses cards back */
            self.send().direct(&caller, &card_id, card_nonce, &self.blockchain().get_sc_balance(&card_id, card_nonce), &[]);
        }

        /*  For each SFT in
           the cards storage */
        for e in self.cards(&booster_id, booster_nonce, rarity).iter()
        {
            /*   If this SFT is
               the one to remove */
            if e.id == card_id && e.nonce == card_nonce
            {
                /* We remove the booster ID */
                self.cards(&booster_id, booster_nonce, rarity).swap_remove(&e);

                /* We end the
                    function  */
                return;
            }
        }
    }

    /****************************************************************************************
     * ~remove_all_cards~                                                                   *
     * ------------------                                                                   *
     * Used by the owner and the SC to remove a card                                        *
     * @param : (TokenIdentifier) booster_id    : the booster ID to remove the card from    *
     * @param : (u64)             booster_nonce : the booster nonce to remove the card from *
     * @param : (nool)            claim         : should the SC send you back all the SFT ? *
     ****************************************************************************************/
    #[only_owner]
    #[endpoint(removeAllCards)]
    fn remove_all_cards(&self, booster_id: TokenIdentifier, booster_nonce: u64, claim: bool)
    {

        /* If claim is true */
        if claim
        {
            /* We claim theses cards back */
            self.claim_booster(booster_id, booster_nonce);

            /* We end the
                function  */
            return;
        }
        
        /* We remove all Common cards
               for such a Booster     */
        self.cards(&booster_id, booster_nonce, Rarity::Common).clear();

        /* We remove all Uncommon cards
                for such a Booster      */
        self.cards(&booster_id, booster_nonce, Rarity::Uncommon).clear();

        /* We remove all Rare cards
                for such a Booster    */
        self.cards(&booster_id, booster_nonce, Rarity::Rare).clear();

        /* We remove all Epic cards
                for such a Booster    */
        self.cards(&booster_id, booster_nonce, Rarity::Epic).clear();

        /* We remove all Legendary cards
                for such a Booster       */
        self.cards(&booster_id, booster_nonce, Rarity::Legendary).clear();
    }


    /* *********************************************************************************************************************************************************************************** */


    /*
     *  Storages
     * declaration
     */


    /* Status storage */
    #[view(getStatus)]
    #[storage_mapper("status")]
    fn status(&self) -> SingleValueMapper<Status>;

    /* U64Datas storage */
    #[view(getU64Data)]
    #[storage_mapper("u64Datas")]
    fn u64_datas(&self, key: U64Key) -> SingleValueMapper<u64>;

    /*  All nfts storage, not
         necessary but avoid
       more expensive computing */
    #[view(getAllNft)]
    #[storage_mapper("allNft")]
    fn all_nft(&self) -> LinkedListMapper<NFT<Self::Api>>;

    /* Boosters ID storage */
    #[view(getBoostersID)]
    #[storage_mapper("boostersID")]
    fn boosters_id(&self) -> UnorderedSetMapper<TokenIdentifier>;

    /* Boosters nonce storage */
    #[view(getBoostersNonce)]
    #[storage_mapper("boostersNonce")]
    fn boosters_nonce(&self, booster_id: &TokenIdentifier) -> UnorderedSetMapper<u64>;

    /* Boosters cards quantity storage */
    #[view(getBoostersCardsQuantity)]
    #[storage_mapper("boostersCardsQuantity")]
    fn boosters_cards_quantity(&self, booster_id: &TokenIdentifier, booster_nonce: u64) -> SingleValueMapper<u64>;

    /* Boosters probabilities storage */
    #[view(getBoostersProbabilities)]
    #[storage_mapper("boostersProbabilities")]
    fn boosters_probabilities(&self, booster_id: &TokenIdentifier, booster_nonce: u64) -> SingleValueMapper<Probability>;

    /* Boosters guarantees storage */
    #[view(getBoostersGuarantees)]
    #[storage_mapper("boostersGuarantees")]
    fn boosters_guarantees(&self, booster_id: &TokenIdentifier, booster_nonce: u64) -> LinkedListMapper<u64>;

    /* Guarantees quantity storage,
         not necessary but avoid
         more expensive computing   */ 
    #[view(getBoostersGuaranteesQuantity)]
    #[storage_mapper("boostersGuaranteesQuantity")]
    fn boosters_guarantees_quantity(&self, booster_id: &TokenIdentifier, booster_nonce: u64) -> SingleValueMapper<u64>;

    /* Boosters constraints storage */
    #[view(getBoostersConstraints)]
    #[storage_mapper("boostersConstraints")]
    fn boosters_constraints(&self, booster_id: &TokenIdentifier, booster_nonce: u64) -> SingleValueMapper<Constraints>;

    /* Cards storage */
    #[view(getCards)]
    #[storage_mapper("cards")]
    fn cards(&self, booster_id: &TokenIdentifier, booster_nonce: u64, rarity: Rarity) -> UnorderedSetMapper<NFT<Self::Api>>;


    /* ****************************************************************************************************************** */
}


/* *************************************************************************************************************************************************************************************** */
