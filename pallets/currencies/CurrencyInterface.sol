// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Currency Precompile contract's address.
address constant CURRENCY_PRECOMPILE_ADDRESS = 0x0000000000000000000000000000000000000000;

/// @dev The Currency Precompile contract's instance.
Currency constant CURRENCY_CONTRACT = Currency(CURRENCY_PRECOMPILE_ADDRESS);

/// @title The Currency Precompile Interface
/// @dev The interface through which solidity contracts will interact with pallet-currency.
/// @custom:address 0x0000000000000000000000000000000000000000
interface Currency {
    /// @dev Gets the total supply of a currency.
    /// @return An uint256 representing the total supply of a currency.
    function totalSupply() external view returns (uint256);

    /// @dev Gets balance of an address.
    /// @param owner address The address that owns a currency.
    /// @return An uint256 representing the balance of the owner.
    function balanceOf(address owner) external view returns (uint256);
    
    /// @dev Gets the currency  allowance of an address.
    /// @param owner address The address that owns a currency
    /// @param currency address The currency address
    /// @return An uint256 representing of the currency for the owner.
    function allowance(address owner, address currency) external view returns (uint256);

    /// @dev Gets the name of a currency.
    /// @return A bytes32 array representing the name of a currency.
    function name() external view returns (bytes32);

    /// @dev Gets the symbol of a currency.
    /// @return A bytes32 array representing the symbol of a currency.
    function symbol() external view returns (bytes32);

    /// @dev Gets the decimals of a currency.
    /// @return An uint256 representing the decimals of a currency.
    function decimals() external view returns (uint256);
    
    /// @dev Transfer currency to a specified address
    /// @custom:selector 10059cba
    /// @param receiver address The address that will receive the currency.
    /// @param amount uint256 The amount that will be transferred.
    /// @return true if the transfer was successful, revert otherwise
    function transfer(address receiver, uint256 amount) external returns (bool);
    
    /// @dev Approve currency for transfer.
    /// @param owner The currency owner address.
    /// @param amount uint256 The amount that will be approved.
    /// @return true if the approval was successful, revert otherwise.
    function approve(address owner, uint256 amount) external returns (bool);

    /// @dev Transfer currency from a specified address to another one.
    /// @param sender The currency sender address.
    /// @param receiver The currency receiver address.
    /// @param amount uint256 The amount that will be transferred.
    /// @return true if the transfer was successful, revert otherwise.
    function transferFrom(address sender, address receiver, uint256 amount) external returns (bool);
    
    /// @dev Event emitted when a currency is transferred.
    /// @param sender address The address that transferred the currency.
    /// @param receiver address The address that received the currency.
    /// @param amount uint256 The amount that was transferred.
    event Transferred(address indexed sender, address receiver, uint256 amount);
    
    /// @dev Event emitted when a currency is approved.
    /// @param owner The currency owner address.
    /// @param amount uint256 The amount that was approved.
    event Approved(address indexed owner, uint256 amount);
}
