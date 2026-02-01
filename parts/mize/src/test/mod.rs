
use crate::instance::MizeInstance;

#[test]
fn most_minimal_instance_usage() {
    // so I'm actually starting the effort for a better code base here in the tests
    // i want to write down how you would use the very important instance type
    // and then implement it to make it usable in this way

    // how every mize based program to run on an os should create a mize instance
    // opens the instance the way the user has configured (with env vars, build args, or config
    // files)
    // and we just call it "new()" and not "default_system()" so that it is clear that this is the
    // method that should be used
    let mize = MizeInstance::new();

    // then the most important things you do with your mize instance is to get/set stuff
    let name: MizeItem = mize.get("1/namespace");
    let id = mize.new_item();
    mize.set(id.join("name"), "test-item");

    // or register update callbacks and update
    //mize.on_update("5/name", |delta| println!(delta.new_value()));
    //mize.update("5/name", "other-item")

    // the goal is that also most things about the MizeInstance itself is controlled by
    // get/set/update -ing values at the 0/ paths
}



