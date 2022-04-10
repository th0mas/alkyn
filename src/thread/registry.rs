//! Registry
//! 
extern crate alloc;
use alloc::collections::BTreeMap;

static mut ALKYN_REGISTRY: BTreeMap<&str, usize> = BTreeMap::new();

pub fn set_registry_for_idx(idx: usize, name: &'static str) {
  unsafe {
    let cs = critical_section::acquire();
    ALKYN_REGISTRY.insert(name, idx);
    critical_section::release(cs);
  }
}

pub fn lookup_by_idx(idx: usize) -> Option<&'static str> {
  let res: Option<&'static str>;
  unsafe {
    let cs = critical_section::acquire();
    let lookup = ALKYN_REGISTRY.iter()
      .filter(|(_, i)| i == &&idx )
      .last();

    res = match lookup {
      Some((s, _)) => Some(s.clone()),
      None => None
    };
    critical_section::release(cs);
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