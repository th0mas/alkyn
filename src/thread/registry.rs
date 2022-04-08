//! Registry
//! 
extern crate alloc;
use alloc::collections::BTreeMap;

static mut ALKYN_REGISTRY: BTreeMap<&str, usize> = BTreeMap::new();

pub fn set_registry_for_idx(idx: usize, name: &'static str) {
  unsafe {
    let cs = critical_section::acquire();
    ALKYN_REGISTRY[&name] = idx;
    critical_section::release(cs);
  }
}

pub fn lookup_by_idx(idx: usize) -> &'static str {
  let res: &str;
  unsafe {
    let cs = critical_section::acquire();
    res = ALKYN_REGISTRY[&idx];
    critical_section::release(cs)
  }
  res
}

pub fn lookup_by_name(name: &str) -> Option<usize> {
  let res: Option<usize>;
  unsafe {
    let cs = critical_section::acquire();
    res = ALKYN_REGISTRY.get(name).cloned();
    critical_section::release(cs)
  }
  res
}