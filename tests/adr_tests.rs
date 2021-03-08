use adrs::ADR;

#[test]
fn test_new() {

    let mut adr = ADR::new("./foo");
    println!("{:?}", adr);
}