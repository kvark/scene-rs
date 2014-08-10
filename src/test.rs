use ces;

pub type SimpleComponent = int;
entity! { ces
	simple : SimpleComponent,
}

#[test]
fn test_simple() {
	let mut hub = DataHub::new();
	let ent = hub.add().simple(4).entity;
	let value = hub.simple.get(ent.simple.unwrap());
	assert_eq!(*value, 4);
}
