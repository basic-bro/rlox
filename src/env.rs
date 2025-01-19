

use std::collections::HashMap;

use crate::{eval::Eval, util::RcMut};

pub struct Env {
  enclosing: Option<RcMut<Env>>,
  values: HashMap<String, Eval>
}