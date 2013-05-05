use core::hashmap::linear::{LinearMap};
use core::flate;

mod opcode;

struct Parser {
  source: ~[u8],
  offset: uint
}

struct Ast {
  chunks: ~[~Chunk]
}

struct Chunk {
  kind: ChunkKind,
  size: uint,
  body: ChunkBody
}

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

enum ChunkBody {
  Raw(~[u8]),
  AtomTable(~LinearMap<uint, Atom>),
  ImportTable(~[Import]),
  ExportTable(~[Export]),
  CodeTable(~LabelMap),
  FunTable(~[FunctionItem]),
  LiteralTable(~[~[u8]]),
  Empty
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

type LabelMap = LinearMap<uint, ~[Opcode]>;

struct Opcode {
  opcode: opcode::Opcode,
  args: ~[Arg]
}

struct Arg {
  tag: ArgTag,
  value: ArgValue
}

#[deriving(Eq)]
enum ArgTag {
  U = 0,
  I = 1,
  A = 2,
  X = 3,
  Y = 4,
  F = 5,
  H = 6,
  Z = 7,
  Float,
  List,
  Fr,
  Lit
}

enum ArgValue {
  IntVal(i64),
  FloatVal(f64),
  AllocList(~[AllocListItem])
}

enum AllocKind {
  WordAlloc,
  FloatAlloc,
  LiteralAlloc
}

struct AllocListItem {
  kind: AllocKind,
  value: i64
}

struct FunctionItem {
  fun: u32,
  atom: u32,
  label: u32,
  index: u32,
  num_free: u32,
  old_uniq: u32
}

impl Ast {
  pub fn to_str(&self) -> ~str {
    return fmt!("%?", self.chunks);
  }
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

  fn read_u64(&mut self) -> u64 {
    return (self.read_u32() as u64 << 32) | (self.read_u32() as u64);
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

  fn parse_atom_chunk(&mut self) -> ChunkBody {
    let atom_count = self.read_u32();

    let mut i = 1;
    let mut atoms = ~LinearMap::new();
    while i <= atom_count {
      self.ensure(1);
      let atom_size: uint = self.read_u8() as uint;
      self.ensure(atom_size);
      let atom = self.slice(atom_size);
      atoms.insert(i as uint, str::from_bytes(atom));
      i += 1;
    }
    return AtomTable(atoms);
  }

  fn parse_import_chunk(&mut self) -> ChunkBody {
    let count = self.read_u32();

    let mut i = 0;
    let mut list = ~[];
    while i < count {
      list.push(Import {
        module: self.read_u32(),
        name: self.read_u32(),
        arity: self.read_u32()
      });
      i += 1;
    }

    return ImportTable(list);
  }

  fn parse_export_chunk(&mut self) -> ChunkBody {
    let count = self.read_u32();

    let mut i = 0;
    let mut list = ~[];
    while i < count {
      list.push(Export {
        name: self.read_u32(),
        arity: self.read_u32(),
        label: self.read_u32()
      });
      i += 1;
    }

    return ExportTable(list);
  }

  fn parse_opcode_arg_i64(&mut self, first: u8) -> i64 {
    if first & 0x8 == 0 {
      // value < 16 (3 bits only)
      return (first >> 4) as i64;
    } else if first & 0x10 == 0 {
      // value <= 2048, occupies 11 bits (3 bits in first byte and whole next byte)
      let next = self.read_u8();
      return ((first & 0xe0) as i64 << 3) | next as i64;
    } else {
      let len = match first >> 5 {
        // Length in the next byte
        7 => {
          let next = self.read_u8();
          self.parse_opcode_arg_i64(next)
        },

        // Small-length integer
        _ => (first >> 5) as i64 + 2
      };

      // TODO: support big numbers
      assert!(len <= i64::bytes as i64);
      let mut i = 0;
      let mut res: i64 = 0;
      let mut sign = false;
      while i < len {
        let byte = self.read_u8();

        // Most significant bit is non-zero - change sign
        if len != 4 && i == 0 && (byte & 0x80) != 0 {
          sign = true;
          res = (res << 8) + (byte & 0x7f) as i64;
        } else {
          res = (res << 8) + byte as i64;
        }

        i += 1;
      }

      if sign {
        res = -res;
      }

      return res;
    }
  }

  fn parse_opcode_arg_alloc(&mut self) -> ~[AllocListItem] {
    // Read number of allocs
    let count = self.parse_opcode_int_arg();

    let mut list = ~[];
    let mut i = 0;
    while i < count {
      let kind = self.parse_opcode_int_arg();
      let value = self.parse_opcode_int_arg();

      list.push(AllocListItem {
        kind: match kind {
          0 => WordAlloc,
          1 => FloatAlloc,
          2 => LiteralAlloc,
          _ => fail!(fmt!("Unexpected alloc list item kind: %?", kind))
        },
        value: value
      });

      i += 1
    }

    return list;
  }

  fn parse_opcode_arg(&mut self) -> Arg {
    let first = self.read_u8();
    let tag: ArgTag  = unsafe { cast::transmute((first & 0x7) as uint) };
    return match tag {
      Z => match first >> 4 {
        // Float
        0 => Arg {
          tag: Float,
          value: FloatVal(unsafe { cast::transmute(self.read_u64()) })
        },

        // List
        1 => Arg { tag: List, value: IntVal((first >> 4) as i64) },

        // Some stuff?
        2 => Arg { tag: Fr, value: copy self.parse_opcode_arg().value },

        // Allocation list
        3 => Arg { tag: U, value: AllocList(self.parse_opcode_arg_alloc()) },

        // Literal
        _ => Arg { tag: Lit, value: IntVal(self.parse_opcode_arg_i64(first)) }
      },
      _ => Arg {
        tag: tag,
        value: IntVal(self.parse_opcode_arg_i64(first))
      },
    }
  }

  fn parse_opcode_int_arg(&mut self) -> i64 {
    let val = copy self.parse_opcode_arg().value;
    return match val {
      IntVal(r) => r,
      _ => fail!(fmt!("Expected int value, got %?", val))
    }
  }

  fn parse_code_chunk(&mut self, size: uint) -> ChunkBody {
    let end = self.offset + size;

    let magic_num = self.read_u32();
    let format_number = self.read_u32();
    let highest_opcode = self.read_u32();

    // Ignore label and function count
    self.offset += 8;

    assert!(magic_num == 16u32);
    assert!(format_number == 0u32);
    assert!(highest_opcode <= opcode::MaxOpcode as u32);

    assert!(self.offset <= end);
    let mut labels = ~LinearMap::new();
    let mut label_id: uint = 1;
    let mut label = ~[];
    while self.offset < end {
      let raw_opcode = self.read_u8();
      if raw_opcode == 0 || raw_opcode >= opcode::MaxOpcode as u8 {
        fail!(fmt!("Unknown opcode met: %?", raw_opcode));
      }

      let opcode: opcode::Opcode = unsafe { cast::transmute(raw_opcode as uint) };
      let arity = opcode::get_arity(opcode);

      let mut args: ~[Arg] = ~[];
      let mut i: uint = 0;
      while i < arity {
        args.push(self.parse_opcode_arg());
        i += 1;
      }

      if opcode == opcode::Label {
        let first = copy args[0];
        assert!(args.len() == 1 && first.tag == U);
        let new_id = match first.value {
          IntVal(id) => id,
          _ => fail!(fmt!("Unexpected level id: %?", first.value))
        } as uint;

        // If label has moved
        if new_id != label_id {
          let no_overwrite = labels.insert(label_id, label);
          assert!(no_overwrite);

          label = ~[];
          label_id = new_id;
        }
      } else {
        label.push(Opcode {
          opcode: opcode,
          args: args
        });
      }
    }

    // Insert trailing code
    if label.len() != 0 {
      let no_overwrite = labels.insert(label_id, label);
      assert!(no_overwrite);
    }

    return CodeTable(labels);
  }

  fn parse_fun_chunk(&mut self) -> ChunkBody {
    let count = self.read_u32();
    let mut i = 0;
    let mut res = ~[];
    while i < count {
      res.push(FunctionItem {
        fun: self.read_u32(),
        atom: self.read_u32(),
        label: self.read_u32(),
        index: self.read_u32(),
        num_free: self.read_u32(),
        old_uniq: self.read_u32()
      });
      i += 1;
    }

    return FunTable(res);
  }

  fn parse_literal_chunk(&mut self, size: uint) -> ChunkBody {
    let data_size = self.read_u32() as uint;
    // Skip zlib header
    self.offset += 2;
    let raw = self.slice(size - 6);
    let data = flate::inflate_bytes(copy raw);
    assert!(data.len() == data_size);

    // Create reader
    let mut p = Parser {
      source: data,
      offset: 0
    };

    // Read each literal
    let mut res: ~[~[u8]] = ~[];
    let count = p.read_u32();
    let mut i = 0;
    while i < count {
      let size = p.read_u32() as uint;
      res.push(p.slice(size));
      i += 1;
    }

    return LiteralTable(res);
  }

  fn parse_chunk(&mut self, kind: ChunkKind, size: uint) -> ~Chunk {
    return ~Chunk {
      kind: kind,
      size: size,
      body: match size {
        0 => Empty,
        _ => match kind {
          Atom => self.parse_atom_chunk(),
          Import => self.parse_import_chunk(),
          Export => self.parse_export_chunk(),
          Local => self.parse_export_chunk(),
          Code => self.parse_code_chunk(size),
          Literal => self.parse_literal_chunk(size),
          Function => self.parse_fun_chunk(),
          _ => Raw(self.slice(size))
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
    let mut chunks = ~[];
    while self.remaining() > 0 {
      let kind = self.parse_chunk_kind();
      let size = self.read_u32() as uint;

      // Parse particular chunk
      let chunk = self.parse_chunk(kind, size);
      chunks.push(chunk);

      // Account padding
      let m = self.offset % 4;
      if m != 0 {
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

#[cfg(test)]
#[test]
fn test_parse_code_chunk() {
  let mut p = Parser {
    source: ~[
      0, 0, 0, 16, // magic
      0, 0, 0, 0, // format
      0, 0, 0, 135, // max instruction
      0, 0, 0, 0, // label count
      0, 0, 0, 0, // function count
      1, 16, // label 1
      2, // function_info
      0x10 | 2, // a:1,
      7, 0x40, 0x2b, 0x2d, 0x91, 0x68, 0x72, 0xb0, 0x21, // z:float 13.589
      0x18 | 0, 0x12, 0x34 // u:0x1234
    ],
    offset: 0
  };

  let len = p.source.len();
  let res = p.parse_code_chunk(len);
  let labels = match res {
    CodeTable(r) => r,
    _ => fail!(~"Result should have CodeChunk type")
  };
  let label = labels.get(&1);
  assert!(label.len() == 1);

  let instr = copy label[0];
  assert!(instr.opcode == opcode::Func_info);
  assert!(instr.args.len() == 3);
  assert!(instr.args[0].tag == A);
  assert!(instr.args[1].tag == Float);
  assert!(instr.args[2].tag == U);
  match instr.args[0].value {
    IntVal(x) => assert!(x == 1),
    _ => fail!(~"Non integer value")
  }
  match instr.args[1].value {
    FloatVal(x) => assert!(x == 13.589f64),
    _ => fail!(~"Non float value")
  }
  match instr.args[2].value {
    IntVal(x) => assert!(x == 0x1234),
    _ => fail!(~"Non integer value")
  }
}
