// use std::cell::RefCell;
// use std::ops::Deref;
// use std::rc::Rc;
// use std::sync::Arc;
// use tokio::sync::{Mutex, OnceCell};
// use crate::{core};
// use crate::core::Resource;
//
// #[macro_use]
// lazy_static::lazy_static! {
//     static ref LOCAL_PATH: OnceCell<String> = OnceCell::new();
//     static ref RESOURCE: OnceCell<core::Resource> = OnceCell::new();
//     static ref RESOURCE_INDEX: OnceCell<String> = OnceCell::new();
// }
//
//
// pub fn set_local_path(value: &str) {
//     LOCAL_PATH.set(value.to_string());
// }
//
// pub fn get_local_path() -> &'static str {
//     LOCAL_PATH.get().unwrap()
// }
//
// pub fn set_resource(value: core::Resource) {
//     RESOURCE.set(value);
// }
//
// pub fn get_resource() -> &'static Resource {
//     RESOURCE.get().unwrap()
// }
//
// pub fn set_resource_index(value: &str) {
//     RESOURCE_INDEX.set(value.to_string());
// }
//
// pub fn get_resource_index() -> Option<&'static str> {
//     match RESOURCE_INDEX.get() {
//         None => None,
//         Some(value) => Some(&value)
//     }
// }