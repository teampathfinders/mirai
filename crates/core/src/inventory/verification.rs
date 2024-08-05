use proto::bedrock::InventoryTransaction;

/// Verifies that inventory transactions are legitimate and do not duplicate items

#[derive(Debug)]
pub enum TransactionValidity {
    Valid
}

/// Verifies that an inventory transaction is valid.
/// This ensures that items are not duplicated.
pub fn verify(transaction: &InventoryTransaction) -> TransactionValidity {
    todo!()
}