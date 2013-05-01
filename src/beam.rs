struct Ast;

impl Ast {
  pub fn print(&self) {
    io::println("I'm AST");
  }
}

pub fn parse(source : ~[u8]) -> ~Ast {
  let mut i = 0;
  return ~Ast;
}
