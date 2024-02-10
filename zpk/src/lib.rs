//! Simple secret sum contract.
//!
//! Calculates the sum of secret inputs from multiple parties. The inputs are not revealed.
//!
//! This implementation works in following steps:
//!
//! 1. Initialization on the blockchain.
//! 2. Receival of multiple secret inputs, using the real zk protocol.
//! 3. The contract owner can start the ZK computation.
//! 4. The Zk computation sums all the given inputs.
//! 5. Once the zk computation is complete, the contract will publicize the summed variable.
//! 6. Once the summed variable is public, the contract will also store it in the state,
//!     such that the value can be read by all.
//!

#![allow(unused_variables)]

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;
extern crate pbc_lib;

use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;
use pbc_contract_common::zk::{CalculationStatus, SecretVarId, ZkInputDef, ZkState, ZkStateChange};
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

/// Secret variable metadata. Unused for this contract, so we use a zero-sized struct to save space.
#[derive(ReadWriteState, ReadWriteRPC, Debug)]
struct SecretVarMetadata {}

/// The maximum size of MPC variables.
const BITLENGTH_OF_SECRET_VARIABLES: u32 = 32;

/// The contract's state
///
/// ### Fields:
///
/// * `administrator`: [`Address`], the administrator of the contract.
///
/// * `sum_result`: [`Option<u32>`], place for storing the final result of the zk computation.
#[state]
struct ContractState {
    /// Address allowed to start computation
    administrator: Address,
    /// Will contain the result (sum) when computation is complete
    sum_result: Option<u32>,
}

/// Initializes the contract and bootstrab the contract state.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], initial context.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], initial zk state.
///
/// ### Returns
///
/// The new state object of type [`ContractState`] with the administrator set to the
/// caller of this function.
#[init]
fn initialize(ctx: ContractContext, zk_state: ZkState<SecretVarMetadata>) -> ContractState {
    ContractState {
        administrator: ctx.sender,
        sum_result: None,
    }
}

/// Adds another secret input of size [`BITLENGTH_OF_SECRET_VARIABLES`].
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context of the current call.
///
/// * `state`: [`ContractState`], the current state of the contract.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], the current zk state.
///
/// ### Returns
///
/// The unchanged state, and a ZkInputDef defining the input size.
#[zk_on_secret_input(shortname = 0x40)]
fn add_input(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (
    ContractState,
    Vec<EventGroup>,
    ZkInputDef<SecretVarMetadata>,
) {
    let input_def = ZkInputDef {
        seal: false,
        metadata: SecretVarMetadata {},
        expected_bit_lengths: vec![BITLENGTH_OF_SECRET_VARIABLES],
    };
    (state, vec![], input_def)
}

/// Automatically called when a variable is confirmed on chain.
///
/// Unused for this contract, so we do nothing.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context of the current call.
///
/// * `state`: [`ContractState`], the current state of the contract.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], the current zk state.
///
/// * `inputted_variable`: [`SecretVarId`], the id of the inputted secret variable.
///
/// ### Returns
/// The unchanged contract state.
#[zk_on_variable_inputted]
fn inputted_variable(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    inputted_variable: SecretVarId,
) -> ContractState {
    state
}

/// Start the zk-computation computing the sum of the secret variables. Only callable by the
/// administrator.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context of the current call.
///
/// * `state`: [`ContractState`], the current state of the contract.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], the current zk state.
///
/// ### Returns
///
/// The unchanged state, and a ZkStateChange denoting that the zk-computation should start.
#[action(shortname = 0x01)]
fn compute_sum(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    assert_eq!(
        context.sender, state.administrator,
        "Only administrator can start computation"
    );
    assert_eq!(
        zk_state.calculation_state,
        CalculationStatus::Waiting,
        "Computation must start from Waiting state, but was {:?}",
        zk_state.calculation_state,
    );

    (
        state,
        vec![],
        vec![ZkStateChange::start_computation(vec![SecretVarMetadata {}])],
    )
}

/// Automatically called when the computation is completed
///
/// The only thing we do is to instantly open/declassify the output variables.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context of the current call.
///
/// * `state`: [`ContractState`], the current state of the contract.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], the current zk state.
///
/// * `output_variables`: [`Vec<SecretVarId>`], the id's of the output variables.
///
/// ### Returns
///
/// The unchanged state, and a ZkStateChange opening the output variables.
#[zk_on_compute_complete]
fn sum_compute_complete(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    output_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    (
        state,
        vec![],
        vec![ZkStateChange::OpenVariables {
            variables: output_variables,
        }],
    )
}

/// Automatically called when a variable is opened/declassified.
///
/// We can now read the sum variable, and save it in the contract state.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context of the current call.
///
/// * `state`: [`ContractState`], the current state of the contract.
///
/// * `zk_state`: [`ZkState<SecretVarMetadata>`], the current zk state.
///
/// * `opened_variables`: [`Vec<SecretVarId>`], the id's of the opened variables.
///
/// ### Returns
///
/// The new state with the computed sum, and a ZkStateChange denoting that the zk computation is done.
#[zk_on_variables_opened]
fn open_sum_variable(
    context: ContractContext,
    mut state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    opened_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    assert_eq!(
        opened_variables.len(),
        1,
        "Unexpected number of output variables"
    );
    let sum = read_variable_u32_le(&zk_state, opened_variables.get(0));
    state.sum_result = Some(sum);
    (state, vec![], vec![ZkStateChange::ContractDone])
}

/// Reads a variable's data as an u32.
///
/// ### Parameters:
///
/// * `zk_state`: [`&ZkState<SecretVarMetadata>`], the current zk state.
///
/// * `sum_variable_id`: [`Option<&SecretVarId>`], the id of the secret variable to be read.
///
/// ### Returns
/// The value of the variable as an [`u32`].
fn read_variable_u32_le(
    zk_state: &ZkState<SecretVarMetadata>,
    sum_variable_id: Option<&SecretVarId>,
) -> u32 {
    let sum_variable_id = *sum_variable_id.unwrap();
    let sum_variable = zk_state.get_variable(sum_variable_id).unwrap();
    let mut buffer = [0u8; 4];
    buffer.copy_from_slice(sum_variable.data.as_ref().unwrap().as_slice());
    <u32>::from_le_bytes(buffer)
}
