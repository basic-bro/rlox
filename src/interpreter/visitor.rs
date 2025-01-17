///////////////////////////////////////////////
// private module rlox::interpreter::visitor //
///////////////////////////////////////////////


//////////////////
// declarations //
//////////////////

#[derive(PartialEq)]
pub enum VisitorControl {
  VisitChildren,
  SkipChildren
}

pub enum MapFolderState<T, E> {
  Complete( Result<T, E> ),
  Incomplete
}
