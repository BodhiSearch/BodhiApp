#[derive(errmeta_derive::ErrorMeta)]
union AUnion {
  msg: i32,
  code: u32,
}

fn main() {}
