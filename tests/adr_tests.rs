use adrs::ADR;

// #[test]
fn test_new() {
    let adr = ADR::new().title("My New ADR").render();
     dbg!(adr);
}
