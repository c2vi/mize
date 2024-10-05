
use tracing_subscriber::registry::Data;

use crate::item::IntoItemData;

use super::*;

#[test]
fn test_get_module_hash() -> MizeResult<()> {
    let mut instance = Instance::empty()?;

    use crate::platform::os::get_module_hash;
    let selector_toml = r#"
        mize_version = "0.0.0"
        system = "x86_64-unknown-linux-gnu"
        toolchain_version = "rustc 1.78.0 (9b00956e5 2024-04-29)"
    "#;

    instance.set_blocking("0/config/selector", ItemData::from_toml(&selector_toml)?)?;

    let hash = get_module_hash(&mut instance, "String", ItemData::new())?;

    test_println!("hash: {}", hash);

    assert_eq!(hash, "e96edf60216688314efc263c1aa8d6ce".to_owned());

    Ok(())
}

#[test]
fn test_set_store_path() -> MizeResult<()> {
    let instance = Instance::empty()?;

    let store_path = "/home/me/.mize".to_owned();

    let config = instance.get("0/config")?.as_data_full()?;

    test_println!("data at 0/config: {}", config);

    instance.set_blocking("0/config/store_path", store_path.clone().into_item_data())?;

    let store_path_new = instance.get("0/config/store_path")?.value_string()?;

    assert_eq!(store_path_new, store_path);

    Ok(())
}

#[test]
fn test_set_sub_path() -> MizeResult<()> {
    let instance = Instance::empty()?;

    let data = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "inner"
    "#)?;

    let data_two = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "new inner"
    "#)?;

    let item = instance.new_item()?;
    instance.set_blocking(item.id(), data.clone())?;

    assert_eq!(item.as_data_full()?, data);

    // try to set a sub path of item
    instance.set_blocking(vec![item.id().store_part(), "config", "test", "inner"], "new inner".into_item_data())?;

    // the contents of item should then be like data_two
    println!("item with id '{}': {}", item.id(), instance.get(vec![item.id().store_part()])?.as_data_full()?);
    assert_eq!(instance.get(vec![item.id().store_part(), "config", "test", "inner"])?.as_data_full()?, ItemData::from_string("new inner"));
    assert_eq!(item.as_data_full()?, data_two);

    Ok(())
}

#[test]
fn test_itemdata_set_path() -> MizeResult<()> {
    let mut data = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "inner"
    "#)?;

    let data_two = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "new inner"
    "#)?;

    data.set_path(vec!["config", "test", "inner"], ItemData::from_string("new inner"))?;

    assert_eq!(data, data_two);

    Ok(())
}

#[test]
fn test_itemdata_merge() -> MizeResult<()> {
    let mut data = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "inner"
    "#)?;

    let data_two = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "new inner"
    "#)?;

    assert_eq!(data.get_path(vec!["config", "test", "inner"])?, ItemData::from_string("inner"));

    data.merge(data_two.clone());

    assert_eq!(data, data_two);

    assert_eq!(data.get_path(vec!["config", "test", "inner"])?, ItemData::from_string("new inner"));

    Ok(())
}

#[test]
fn test_item_merge() -> MizeResult<()> {
    let instance = Instance::empty()?;

    let mut item = instance.new_item()?;

    // here the item should contain "empty" data
    assert_eq!(item.as_data_full()?, ItemData::new());

    let data = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "inner"
    "#)?;

    let data_two = ItemData::from_toml(r#"
        [config]
        hi = "hello from config/hi"

        [config.test]
        inner = "new inner"
    "#)?;

    item.merge(data.clone())?;

    // now it should contain data
    assert_eq!(item.as_data_full()?, data);

    let mut sub_item = instance.get(vec![item.id().store_part(), "config", "test", "inner"])?;

    // the sub item should have the string "inner"
    assert_eq!(sub_item.as_data_full()?, ItemData::from_string("inner"));

    sub_item.merge(ItemData::from_string("new inner"));

    assert_eq!(sub_item.as_data_full()?, ItemData::from_string("new inner"));

    // also the item should be updated
    assert_eq!(item.as_data_full()?, data_two);


    Ok(())
}

/*
#[test]
#[should_panic(expected = "correct panic")]
fn test_cant_set_non_existent_item() -> () {
    let instance = Instance::empty().expect("wrong panic");
    instance.set_blocking("5", "hello world".into_item_data()).expect("correct panic");
}
*/



