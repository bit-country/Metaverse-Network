@justinphamnz  This is my initial design for the country governance pallet. Let me know how it can be improved. Also what type of proposals are we going to consider valid? 
# bc-governance pallet design:
* Calls
    - propose - works only if  (1) account have enough funds for deposit (2) proposal_parameters are <= max_params_per_proposal, (3) changes are in defined scope(match the some of the CountryParameter enum values) (4) proposer is resident of the country, and (5) there is space in the proposal queue; sets the start of the referendum to earliest of current_block + min_launch_period or one block after the end of last referendum voting period
      - account: AccountId
      - country: CountryId
      - balance: Balance
      - proposal_parameters: Vec [CountryParameter]
      - proposal_description: Vec [u8]
    - cancel_proposal - works if you have created the proposal
      - account: AccountId
      - proposal: ProposalId
      - country: CountryId
    - vote - works only if the account is a resident of a referendum's country
      - origin: AccountId
      - referendum: ReferendumId
      - vote_aye: bool
    - remove_vote - works only if the account is a resident of a referendum's country and already voted in the referendum 
      - account: AccountId
      - referendum: ReferendumId
    - update_referendum_parameters - works only if you are the country owner
      - owner: AccountId
      - country: CountryId
      - new_referendum_parameters: ReferendumParameters
    - fast_track_proposal - works only if you are the country owner; puts proposal on top of the referendum queue
      - owner: AccountId
      - proposal: ProposalId
      - country: CountryId
    - emergency_cancel_referendum - can be activated only by account with given privileges; the account can't cancel a vote about the removal of these privileges
      - origin: AccountId
      - referendum: ReferendumId
* Storages
  - Proposals: double_map (CountryId, ProposalId) => Option(ProposalInfo)
  - NextProposalId: ProposalId
  - ReferendumInfoOf: map ReferendumId => ReferendumInfo
  - NextReferendumId: ReferendumId
  - ReferendumParametersOf: map CountryId => ReferendumParameters
  - VotingOf: map: AccountId => VotingRecord
  - DepositOf: map ProposalId => Option<(AccountId,Balance)>
  
* Types
  - struct VotingRecord 
    - votes: Vec [(ReferendumId,Vote)]
  - struct Vote
    - who: AccountId
    - aye: bool
    - balance: Balance
  - enum ReferendumInfo
    - Ongoing(ReferendumStatus)
    - Finished(result, end_of_referendum_block)
  - struct ReferendumStatus
    - end: BlockNumber
    - country: CountryId 
    - proposal: ProposalId
    - threshold: VoteThreshold
    - tally: Tally
  - struct Tally
    - ayes: Balance,
    - nays: Balance,
    - turnout: u64
  - struct ProposalInfo
    - proposed_by: AccountId
    - parameters: Vec [CountryParameter]
    - description: Vec [u8] // link to proposal description
    - referendum_launch_block: BlockNumber
  - struct ReferendumParameters
    - voting_threshold: VoteThreshold 
    - min_proposal_launch_period: ProposalLaunchPeriod // number of blocks
    - voting_period: VotingPeriod // number of blocks
    - enactment_period: EnactmentPeriod // number of blocks
    - max_params_per_proposal: u8
    - proposals_per_country: u8
  - enum CountryParameter - parameters will be enum values in the format parameter_name(parameter_value_type); **TO BE DISCUSSED!:** enum values
  - enum VoteThreshold 
    - StandardQualifiedMajority, // 72%+ 72%+ qorum
    - ReinforcedQualifiedMajority, // 55%+ 65%+ qorum
    - TwoThirdsSupermajority, // 66%+
    - ThreeFifthsSupermajority, // 60%+
    - AbsoluteMajority, // 50%+
    - RelativeMajorty, // Most votes

  
* Events
  - RefrendumParametersUpdated
    - country: CountryId
  - ProposalSubmitted
     - account: AccountId
     - country: CountryId
     - proposal: ProposalId
  - ProposalCancelled
    - account: AccountId
    - proposal: ProposalId
  - ProposalFastTracked
    - proposal: ProposalId
  - ReferendumStarted
    - referendum: ReferendumId
    - voting_threshold: VoteThreshold
  - VoteRecorded
    - origin: AccountId
    - referendum: ReferendumId
    - vote: Vote
  - VoteRemoved
    - origin: AccountId
    - referendum: ReferendumId
  - ReferendumPassed
    - ReferendumId
  - ReferendumNotPassed
    - ReferendumId
  - ReferendumCancelled
    - ReferendumId
  

* Hooks
  - on_initialize
    - start proposal if the current block = end of the proposal launch period
  - on_finalize 
    - trigger referendum result event if the voting period is over
      - return deposited balance to account
      - remove the referendum from the queue if it does not pass 
      - if the referendum passes then enact the proposal
  
* Other Functions
  - update_referendum_tally
    - referendum - ReferendumId
    - vote: Vote
    - remove: bool
  - enact_proposal - schedule country parameters update after the enactment period of a proposal is over
    - proposal: ProposalId 
  - update_country_parameters - unlocks reserved balances as well
    - country: CountryId
    - new_parameters: Vec[CountryParameter]