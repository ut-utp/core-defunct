use embedded_hal::adc::OneShot;
use embedded_hal_mock::adc::Mock;
use embedded_hal_mock::adc::Transaction;
use embedded_hal_mock::adc::{MockChan0, MockChan1};

fn main() {
    // Configure expectations: expected input channel numbers and values returned by read operations
    let expectations = [
        Transaction::read(0, 0xab),
        Transaction::read(1, 0xabcd)
    ];
    let mut adc = Mock::new(&expectations);

    // Reading
    assert_eq!(0xab, adc.read(&mut MockChan0 {}).unwrap());
    assert_eq!(0xabcd, adc.read(&mut MockChan1 {}).unwrap());

    // Finalise expectations
    adc.done();
}