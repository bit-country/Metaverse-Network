// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Metaverse Precompile contract's address.
address constant METAVERSE_PRECOMPILE_ADDRESS = 0x0101010101010101010000000000000000000000;

/// @dev The Metaverse Precompile contract's instance.
Metaverse constant METAVERSE_CONTRACT = Metaverse(METAVERSE_PRECOMPILE_ADDRESS);

/// @title  The Metaverse Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-metaverse.
/// @custom:address 0x0101010101010101010000000000000000000000
interface Metaverse {
    /// @dev Gets the owner of the specified metaverse fund address.
    /// @custom:selector 31a642be
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return The address of the metaverse owner.
    function getMetaverseOwner(uint256 metaverse_id) external view returns (address);

    /// @dev Gets the metadata of the specified metaverse fund address.
    /// @custom:selector dd58ec4e
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return A bytes array representing the metaverse metadata.
    function getMetaverseMetadata(uint256 metaverse_id) external view returns (bytes32);
    
    /// @dev Gets the balance of the specified metaverse fund address.
    /// @custom:selector a9c444a8
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return An uint256 representing the class fund balance.
    function getMetaverseFundBalance(uint256 metaverse_id) external view returns (uint256);
    
    /// @dev Create metaverse
    /// @custom:selector b9059cba
    /// @param metadata bytes The metadata of the metaverse.
    /// @return true if the mint was successful, revert otherwise.
    function createMetaverse(bytes32 metadata) external returns (bool);
    
    /// @dev Withdraw from metaverse fund
    /// @custom:selector 34c05f62
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return true if the withdraw was successful, revert otherwise.
    function withdrawFromMetaverseFund(uint256 metaverse_id) external returns (bool);
    
    /// @dev Transfer metaverse to specified address
    /// @custom:selector f063c7b9
    /// @param to address The new metaverse owner address.
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @return true if the transfer was successful, revert otherwise.
    function transferMetaverse(address to, uint256 metaverse_id) external returns (bool);

    /// @dev Event emitted when a metaverse is created.
    /// @param metaverse_id uint256 The ID of the metaverse.
    event metaverseCreated(uint256 indexed metaverse_id);

    /// @dev Event emitted when a metaverse is transferred.
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @param owner address The address of the new owner
    event metaverseTransfered(uint256 indexed metaverse_id, addreess owner);
    
    /// @dev Event emitted when withdraw from metaverse fund is successful.
    /// @param metaverse_id uint256 The ID of the metaverse.
    /// @param balance uint256 The amount of tokens transferred.
    event metaverseFundWithdraw(uint256 indexed metaverse_id, uint256 balance);
}
