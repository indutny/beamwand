use core::hashmap::linear::{LinearMap};

mod opcode;

enum ChunkKind {
  Atom,
  Export,
  Import,
  Code,
  String,
  Literal,
  Function,
  Attr,
  CInfo,
  Local,
  Abst,
  Line,
  Trace
}

type Atom = ~str;

struct Import {
  module: u32,
  name: u32,
  arity: u32
}

struct Export {
  name: u32,
  arity: u32,
  label: u32
}

enum OpcodeArgKind {
  U = 0,
  I = 1,
  A = 2,
  X = 3,
  Y = 4,
  F = 5,
  H = 6,
  Z = 7
}

struct OpcodeArg {
  kind: OpcodeArgKind,
  value: uint
}

struct Opcode {
  opcode: opcode::Opcode,
  args: ~[~OpcodeArg]
}

type LabelMap = LinearMap<uint, ~[~Opcode]>;

enum ChunkBody {
  Raw(~[u8]),
  AtomChunk(~LinearMap<uint, Atom>),
  ImportChunk(~[~Import]),
  ExportChunk(~[~Export]),
  CodeChunk(~LabelMap),
  Empty
}

struct Chunk {
  kind: ChunkKind,
  size: uint,
  body: ~ChunkBody
}

struct Ast {
  chunks: ~[~Chunk]
}

impl Ast {
  pub fn to_str(&self) -> ~str {
    return fmt!("%?", self.chunks);
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

  fn read_u8(&mut self) -> u8 {
    self.ensure(1);
    let r = self.get(0);
    self.offset += 1;
    return r;
  }

  fn read_u16(&mut self) -> u16 {
    self.ensure(2);
    let r = (self.get(0) as u16 << 8) | (self.get(1) as u16);
    self.offset += 2;
    return r;
  }

  fn read_u32(&mut self) -> u32 {
    return (self.read_u16() as u32 << 16) | (self.read_u16() as u32);
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
    } else if self.try_match4("LitT") {
      return Literal;
    } else if self.try_match4("FunT") {
      return Function;
    } else if self.try_match4("Attr") {
      return Attr;
    } else if self.try_match4("CInf") {
      return CInfo;
    } else if self.try_match4("LocT") {
      return Local;
    } else if self.try_match4("Abst") {
      return Abst;
    } else if self.try_match4("Line") {
      return Line;
    } else if self.try_match4("Trac") {
      return Trace;
    }
    fail!(fmt!("Failed to parse chunk kind: %?", str::from_bytes(self.slice(4))));
  }

  fn parse_atom_chunk(&mut self) -> ~ChunkBody {
    let atom_count = self.read_u32();

    let mut i = 1;
    let mut atoms: ~LinearMap<uint, Atom> = ~LinearMap::new();
    while i <= atom_count {
      self.ensure(1);
      let atom_size: uint = self.read_u8() as uint;
      self.ensure(atom_size);
      let atom = self.slice(atom_size);
      atoms.insert(i as uint, str::from_bytes(atom));
      i += 1;
    }
    return ~AtomChunk(atoms);
  }

  fn parse_import_chunk(&mut self) -> ~ChunkBody {
    let count = self.read_u32();

    let mut i = 0;
    let mut list: ~[~Import] = ~[];
    while i < count {
      list.push(~Import {
        module: self.read_u32(),
        name: self.read_u32(),
        arity: self.read_u32()
      });
      i += 1;
    }

    return ~ImportChunk(list);
  }

  fn parse_export_chunk(&mut self) -> ~ChunkBody {
    let count = self.read_u32();

    let mut i = 0;
    let mut list: ~[~Export] = ~[];
    while i < count {
      list.push(~Export {
        name: self.read_u32(),
        arity: self.read_u32(),
        label: self.read_u32()
      });
      i += 1;
    }

    return ~ExportChunk(list);
  }

  fn parse_opcode_arg(&mut self) -> ~OpcodeArg {
    fail!(~"Not implemented yet");
  }

  fn parse_code_chunk(&mut self, size: uint) -> ~ChunkBody {
    let end = self.offset + size;

    let magic_num = self.read_u32();
    let format_number = self.read_u32();
    let highest_opcode = self.read_u32();
    assert!(magic_num == 16u32);
    assert!(format_number == 0u32);
    assert!(highest_opcode <= opcode::MaxOpcode as u32);

    assert!(self.offset <= end);
    let mut labels: ~LabelMap = ~LinearMap::new();
    let mut label: ~[~Opcode] = ~[];
    while (self.offset <= end) {
      let raw_opcode = self.read_u32();
      if raw_opcode == 0 || raw_opcode >= opcode::MaxOpcode as u32 {
        fail!(fmt!("Unknown opcode met: %?", raw_opcode));
      }

      let opcode: opcode::Opcode = unsafe { cast::transmute(raw_opcode as uint) };
      let arity = opcode::get_arity(opcode);
      io::println(fmt!("%? %?", opcode, arity));

      let mut args: ~[~OpcodeArg] = ~[];
      let mut i: uint = 0;
      while i < arity {
        args.push(self.parse_opcode_arg());
        i += 1;
      }

      label.push(~Opcode {
        opcode: opcode,
        args: args
      });
    }

    return ~CodeChunk(labels);
  }

  fn parse_chunk(&mut self, kind: ChunkKind, size: uint) -> ~Chunk {
    return ~Chunk {
      kind: kind,
      size: size,
      body: match size {
        0 => ~Empty,
        _ => match kind {
          Atom => self.parse_atom_chunk(),
          Import => self.parse_import_chunk(),
          Export => self.parse_export_chunk(),
          Local => self.parse_export_chunk(),
          Code => self.parse_code_chunk(size),
          _ => ~Raw(self.slice(size))
        }
      }
    }
  }

  fn run(&mut self) -> ~Ast {
    // Parse header
    self.match4(~"FOR1");
    let form_len = self.read_u32();
    assert!(form_len <= self.remaining() as u32);
    self.match4(~"BEAM");

    // Parse chunks
    let mut chunks: ~[~Chunk] = ~[];
    while self.remaining() > 0 {
      let kind = self.parse_chunk_kind();
      let size = self.read_u32() as uint;

      // Parse particular chunk
      let chunk = self.parse_chunk(kind, size);
      chunks.push(chunk);

      // Account padding
      let m = self.offset % 4;
      if (m != 0) {
        self.offset += 4 - m;
      }
    }

    return ~Ast {
      chunks: chunks
    };
  }
}

pub fn parse(source: ~[u8]) -> ~Ast {
  let mut p = Parser {
    source: source,
    offset: 0
  };
  return p.run();
}
