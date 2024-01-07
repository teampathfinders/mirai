#[test]
fn order_channel() {
    let mut channel = OrderChannel::new();

    let mut test_frame = Frame::default();
    test_frame.order_index = 0;
    assert!(channel.insert(test_frame).is_some());

    let mut test_frame = Frame::default();
    test_frame.order_index = 2;
    assert!(channel.insert(test_frame).is_none());

    let mut test_frame = Frame::default();
    test_frame.order_index = 1;
    let output = channel.insert(test_frame).unwrap();

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].order_index, 1);
    assert_eq!(output[1].order_index, 2);
}