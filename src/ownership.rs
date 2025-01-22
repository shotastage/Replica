use std::collections::HashMap;
use crate::ast::OwnershipInfo;

pub struct OwnershipChecker {
    symbol_table: HashMap<String, OwnershipInfo>,
}

impl OwnershipChecker {
    pub fn new() -> Self {
        OwnershipChecker {
            symbol_table: HashMap::new(),
        }
    }

    pub fn check_move(&mut self, _var_name: &str) -> Result<(), String> {
        // TODO: Implement ownership movement checking
        todo!()
    }

    pub fn check_copy(&mut self, _from: &str, _to: &str) -> Result<(), String> {
        // TODO: Implement copy validation
        todo!()
    }
}
