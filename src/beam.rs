enum ChunkKind {
  Atom,
  Export,
  Import,
  Code,
  String,
  Attr,
  CInfo,
  Local,
  Abst,
  Line,
  Trace
}

enum ChunkBody {
  Raw(~[u8])
}

struct Chunk {
  kind: ChunkKind,
  size: uint,
  body: ChunkBody
}

struct Ast {
  chunks: ~[Chunk]
}

impl Ast {
  pub fn print(&self) {
    io::println("I'm AST");
  }
}

struct Parser {
  source: ~[u8],
  offset: uint
}

impl Parser {
  #[inline(always)]
  fn ensure(&self, n: uint) {
    if self.offset + n > self.source.len() {
      fail!(fmt!("Failed to ensure %? bytes", n));
    }
  }

  #[inline(always)]
  fn get(&self, off: uint) -> u8 {
    self.source[self.offset + off]
  }

  #[inline(always)]
  fn remaining(&self) -> uint {
    return self.source.len() - self.offset;
  }

  fn try_match4(&mut self, s: &str) -> bool {
    assert!(s.len() == 4);
    self.ensure(4);

    if self.get(0) == s[0] && self.get(1) == s[1] &&
       self.get(2) == s[2] && self.get(3) == s[3] {
      self.offset += 4;
      return true;
    }
    return false;
  }

  fn match4(&mut self, s: &str) {
    if !self.try_match4(s) {
      fail!(fmt!("Failed to match: %s", s));
    }
  }

  fn read_u16(&self, off: uint) -> u16 {
    self.ensure(off + 2);
    return (self.get(off) as u16 << 8) | (self.get(off + 1) as u16);
  }

  fn read_u32(&self, off: uint) -> u32 {
    return (self.read_u16(off) as u32 << 16) | (self.read_u16(off + 2) as u32);
  }

  fn slice(&mut self, size: uint) -> ~[u8] {
    self.ensure(size);
    let res = vec::slice(self.source, self.offset, self.offset + size).to_vec();
    self.offset += size;
    return res;
  }

  fn parse_chunk_kind(&mut self) -> ChunkKind {
    if self.try_match4("Atom") {
      return Atom;
    } else if self.try_match4("ExpT") {
      return Export;
    } else if self.try_match4("ImpT") {
      return Import;
    } else if self.try_match4("Code") {
      return Code;
    } else if self.try_match4("StrT") {
      return String;
    } else if self.try_match4("Attr") {
      return Attr;
    } else if self.try_match4("CInf") {
      return CInfo;
    } else if self.try_match4("LocT") {
      return Local
    } else if self.try_match4("Abst") {
      return Abst
    } else if self.try_match4("Line") {
      return Line
    } else if self.try_match4("Trac") {
      return Trace
    }
    fail!(~"Failed to parse chunk kind");
  }

  fn parse_chunk(kind: ChunkKind, size: uint, body: ~[u8]) -> ~Chunk {
    return ~Chunk {
      kind: kind,
      size: size,
      body: match (kind) {
        Atom => {
          Raw(body)
        },
        _ => Raw(body)
      }
    }
  }

  fn run(&mut self) {
    // Parse header
    self.match4(~"FOR1");
    let form_len = self.read_u32(0);
    self.offset += 4;
    assert!(form_len <= self.remaining() as u32);
    self.match4(~"BEAM");

    // Parse chunks
    let mut chunks : ~[~Chunk] = ~[];
    while self.remaining() > 0 {
      self.ensure(4);
      let kind = self.parse_chunk_kind();
      let size = self.read_u32(0) as uint;
      self.offset += 4;

      // Parse particular chunk
      let chunk = Parser::parse_chunk(kind, size, self.slice(size));
      vec::push(&mut chunks, chunk);

      // Account padding
      let m = self.offset % 4;
      if (m != 0) {
        self.offset += 4 - m;
      }
    }
    io::println(fmt!("%?", chunks));
  }
}

pub fn parse(source: ~[u8]) -> ~Ast {
  let mut p = Parser {
    source: source,
    offset: 0
  };
  p.run();
  return ~Ast {
    chunks: ~[]
  };
}
