extern crate "codius-sandbox-core" as sandbox;

fn main() {
  let mut sbox = sandbox::Sandbox::new();
  sbox.spawn (&["/bin/true"]);
}
