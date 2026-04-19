// Auto-generated contract assertions from YAML — DO NOT EDIT.
// Zero cost in release builds (debug_assert!).
// Regenerate: pv codegen contracts/ -o src/generated_contracts.rs
// Include:   #[macro_use] #[allow(unused_macros)] mod generated_contracts;

// Auto-generated from contracts/shell-execution-v1.yaml — DO NOT EDIT
// Contract: shell-execution-v1

/// Preconditions for equation `config_validation`.
/// Call at function entry: `contract_pre_config_validation!(input_expr)`
macro_rules! contract_pre_config_validation {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
    }};
}

/// Invariants for equation `config_validation`.
/// Check after computation: `contract_inv_config_validation!(result_expr)`
macro_rules! contract_inv_config_validation {
    () => {{}};
    ($result:expr) => {{
        let _contract_result = &$result;
    }};
}

/// Preconditions for equation `parser_correctness`.
/// Domain-specific. Call: `contract_pre_parser_correctness!(slice_expr)`
macro_rules! contract_pre_parser_correctness {
    () => {{}};
    ($input:expr) => {{
        let _pv_input = &$input;
        debug_assert!(
            _pv_input.len() <= 1_048_576,
            "Contract parser_correctness: precondition violated — input.len() <= 1_048_576"
        );
    }};
}

/// Invariants for equation `parser_correctness`.
/// Check after computation: `contract_inv_parser_correctness!(result_expr)`
macro_rules! contract_inv_parser_correctness {
    () => {{}};
    ($result:expr) => {{
        let _contract_result = &$result;
    }};
}

/// Preconditions for equation `startup_budget`.
/// Call at function entry: `contract_pre_startup_budget!(input_expr)`
macro_rules! contract_pre_startup_budget {
    () => {{}};
    ($input:expr) => {{
        let _contract_input = &$input;
    }};
}

/// Invariants for equation `startup_budget`.
/// Check after computation: `contract_inv_startup_budget!(result_expr)`
macro_rules! contract_inv_startup_budget {
    () => {{}};
    ($result:expr) => {{
        let _contract_result = &$result;
    }};
}

// Total: 1 preconditions, 0 postconditions, 0 invariants from 1 contracts
