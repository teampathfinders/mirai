use crate::Replicator;

#[tokio::test]
async fn test_replicator() {
    let replicator = Replicator::new().await.unwrap();

}