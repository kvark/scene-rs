pub type SimpleComponent = int;
world! { ces (()),
    simple : SimpleComponent,
}

#[test]
fn test_simple() {
    let mut hub = Components::new();
    let ent = hub.add().simple(4).entity;
    let value = hub.simple.get(ent.simple.unwrap());
    assert_eq!(*value, 4);
}
