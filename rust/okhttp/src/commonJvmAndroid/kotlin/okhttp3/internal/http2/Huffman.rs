use std::io::{self, Read, Write};
use std::sync::Arc;
use once_cell::sync::Lazy;

// This class was originally composed from the following classes in
// [Twitter Hpack][twitter_hpack].
//
//  * `com.twitter.hpack.HuffmanEncoder`
//  * `com.twitter.hpack.HuffmanDecoder`
//  * `com.twitter.hpack.HpackUtil`
//
// [twitter_hpack]: https://github.com/twitter/hpack
pub struct Huffman;

// Appendix C: Huffman Codes
// http://tools.ietf.org/html/draft-ietf-httpbis-header-compression-12#appendix-B
const CODES: [i32; 256] = [
    0x1ff8, 0x7fffd8, 0xfffffe2, 0xfffffe3, 0xfffffe4, 0xfffffe5, 0xfffffe6, 0xfffffe7, 0xfffffe8, 0xffffea,
    0x3ffffffc, 0xfffffe9, 0xfffffea, 0x3ffffffd, 0xfffffeb, 0xfffffec, 0xfffffed, 0xfffffee, 0xfffffef, 0xffffff0,
    0xffffff1, 0xffffff2, 0x3ffffffe, 0xffffff3, 0xffffff4, 0xffffff5, 0xffffff6, 0xffffff7, 0xffffff8, 0xffffff9,
    0xffffffa, 0xffffffb, 0x14, 0x3f8, 0x3f9, 0xffa, 0x1ff9, 0x15, 0xf8, 0x7fa,
    0x3fa, 0x3fb, 0xf9, 0x7fb, 0xfa, 0x16, 0x17, 0x18, 0x0, 0x1,
    0x2, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x5c, 0xfb,
    0x7ffc, 0x20, 0xffb, 0x3fc, 0x1ffa, 0x21, 0x5d, 0x5e, 0x5f, 0x60,
    0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
    0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0xfc, 0x73,
    0xfd, 0x1ffb, 0x7fff0, 0x1ffc, 0x3ffc, 0x22, 0x7ffd, 0x3, 0x23, 0x4,
    0x24, 0x5, 0x25, 0x26, 0x27, 0x6, 0x74, 0x75, 0x28, 0x29,
    0x2a, 0x7, 0x2b, 0x76, 0x2c, 0x8, 0x9, 0x2d, 0x77, 0x78,
    0x79, 0x7a, 0x7b, 0x7ffe, 0x7fc, 0x3ffd, 0x1ffd, 0xffffffc, 0xfffe6, 0x3fffd2,
    0xfffe7, 0xfffe8, 0x3fffd3, 0x3fffd4, 0x3fffd5, 0x7fffd9, 0x3fffd6, 0x7fffda, 0x7fffdb, 0x7fffdc,
    0x7fffdd, 0x7fffde, 0xffffeb, 0x7fffdf, 0xffffec, 0xffffed, 0x3fffd7, 0x7fffe0, 0xffffee, 0x7fffe1,
    0x7fffe2, 0x7fffe3, 0x7fffe4, 0x1fffdc, 0x3fffd8, 0x7fffe5, 0x3fffd9, 0x7fffe6, 0x7fffe7, 0xffffef,
    0x3fffda, 0x1fffdd, 0xfffe9, 0x3fffdb, 0x3fffdc, 0x7fffe8, 0x7fffe9, 0x1fffde, 0x7fffea, 0x3fffdd,
    0x3fffde, 0xfffff0, 0x1fffdf, 0x3fffdf, 0x7fffeb, 0x7fffec, 0x1fffe0, 0x1fffe1, 0x3fffe0, 0x1fffe2,
    0x7fffed, 0x3fffe1, 0x7fffee, 0x7fffef, 0xfffea, 0x3fffe2, 0x3fffe3, 0x3fffe4, 0x7ffff0, 0x3fffe5,
    0x3fffe6, 0x7ffff1, 0x3ffffe0, 0x3ffffe1, 0xfffeb, 0x7fff1, 0x3fffe7, 0x7ffff2, 0x3fffe8, 0x1ffffec,
    0x3ffffe2, 0x3ffffe3, 0x3ffffe4, 0x7ffffde, 0x7ffffdf, 0x3ffffe5, 0xfffff1, 0x1ffffed, 0x7fff2, 0x1fffe3,
    0x3ffffe6, 0x7ffffe0, 0x7ffffe1, 0x3ffffe7, 0x7ffffe2, 0xfffff2, 0x1fffe4, 0x1fffe5, 0x3ffffe8, 0x3ffffe9,
    0xffffffd, 0x7ffffe3, 0x7ffffe4, 0x7ffffe5, 0xfffec, 0xfffff3, 0xfffed, 0x1fffe6, 0x3fffe9, 0x1fffe7,
    0x1fffe8, 0x7ffff3, 0x3fffea, 0x3fffeb, 0x1ffffee, 0x1ffffef, 0xfffff4, 0xfffff5, 0x3ffffea, 0x7ffff4,
    0x3ffffeb, 0x7ffffe6, 0x3ffffec, 0x3ffffed, 0x7ffffe7, 0x7ffffe8, 0x7ffffe9, 0x7ffffea, 0x7ffffeb, 0xffffffe,
    0x7ffffec, 0x7ffffed, 0x7ffffee, 0x7ffffef, 0x7fffff0, 0x3ffffee,
];

const CODE_BIT_COUNTS: [u8; 256] = [
    13, 23, 28, 28, 28, 28, 28, 28, 28, 24, 30, 28, 28, 30, 28, 28, 28, 28, 28, 28, 28, 28, 30, 28,
    28, 28, 28, 28, 28, 28, 28, 28, 6, 10, 10, 12, 13, 6, 8, 11, 10, 10, 8, 11, 8, 6, 6, 6, 5, 5,
    5, 6, 6, 6, 6, 6, 6, 6, 7, 8, 15, 6, 12, 10, 13, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    7, 7, 7, 7, 7, 7, 7, 7, 8, 7, 8, 13, 19, 13, 14, 6, 15, 5, 6, 5, 6, 5, 6, 6, 6, 5, 7, 7, 6, 6,
    6, 5, 6, 7, 6, 5, 5, 6, 7, 7, 7, 7, 7, 15, 11, 14, 13, 28, 20, 22, 20, 20, 22, 22, 22, 23, 22,
    23, 23, 23, 23, 23, 24, 23, 24, 24, 22, 23, 24, 23, 23, 23, 23, 21, 22, 23, 22, 23, 23, 24, 22,
    21, 20, 22, 22, 23, 23, 21, 23, 22, 22, 24, 21, 22, 23, 23, 21, 21, 22, 21, 23, 22, 23, 23, 20,
    22, 22, 22, 23, 22, 22, 23, 26, 26, 20, 19, 22, 23, 22, 25, 26, 26, 26, 27, 27, 26, 24, 25, 19,
    21, 26, 27, 27, 26, 27, 24, 21, 21, 26, 26, 28, 27, 27, 27, 20, 24, 20, 21, 22, 21, 21, 23, 22,
    22, 25, 25, 24, 24, 26, 23, 26, 27, 26, 26, 27, 27, 27, 27, 27, 28, 27, 27, 27, 27, 27, 26,
];

#[derive(Clone)]
struct Node {
    children: Option<Vec<Option<Arc<Node>>>>,
    symbol: i32,
    terminal_bit_count: i32,
}

impl Node {
    fn new_internal() -> Self {
        Node {
            children: Some(vec![None; 256]),
            symbol: 0,
            terminal_bit_count: 0,
        }
    }

    fn new_terminal(symbol: i32, bits: i32) -> Self {
        let b = bits & 0x07;
        Node {
            children: None,
            symbol,
            terminal_bit_count: if b == 0 { 8 } else { b },
        }
    }
}

static ROOT: Lazy<Arc<Node>> = Lazy::new(|| {
    let mut root = Node::new_internal();
    // We need a way to mutate the tree during construction.
    // Since the Kotlin code uses a mutable root and adds codes, 
    // we'll simulate this by building the tree in a mutable fashion first.
    
    // In Rust, to avoid complex Arc mutation, we can use a temporary 
    // structure or interior mutability. For the static init, we'll 
    // use a helper function that builds the tree.
    build_huffman_tree()
});

fn build_huffman_tree() -> Arc<Node> {
    // Use a raw pointer or a different structure for construction to avoid Arc overhead
    // but for fidelity to the logic, we'll use a mutable root and a recursive-like approach.
    // Since we are in a Lazy init, we can just build it.
    
    // To allow mutation of the tree during build, we use a temporary mutable root.
    // Because the tree is a DAG/Tree of Nodes, we'll use a simple vector-based 
    // approach or just use RefCell/Mutex if needed. 
    // Actually, the simplest way to mirror the Kotlin `addCode` is to use 
    // a mutable structure and then wrap it in Arc.
    
    struct MutableNode {
        children: Option<Vec<Option<Box<MutableNode>>>>,
        symbol: i32,
        terminal_bit_count: i32,
    }

    impl MutableNode {
        fn new_internal() -> Self {
            MutableNode {
                children: Some(vec![None; 256]),
                symbol: 0,
                terminal_bit_count: 0,
            }
        }
        fn new_terminal(symbol: i32, bits: i32) -> Self {
            let b = bits & 0x07;
            MutableNode {
                children: None,
                symbol,
                terminal_bit_count: if b == 0 { 8 } else { b },
            }
        }
    }

    let mut root = MutableNode::new_internal();

    for i in 0..256 {
        let symbol = i as i32;
        let code = CODES[i];
        let code_bit_count = CODE_BIT_COUNTS[i] as i32;
        
        let terminal = Box::new(MutableNode::new_terminal(symbol, code_bit_count));
        let mut accumulator_bit_count = code_bit_count;
        let mut current_node = &mut root;

        while accumulator_bit_count > 8 {
            accumulator_bit_count -= 8;
            let child_index = ((code as u32 >> accumulator_bit_count) & 0xff) as usize;
            
            if current_node.children.as_mut().unwrap()[child_index].is_none() {
                current_node.children.as_mut().unwrap()[child_index] = Some(Box::new(MutableNode::new_internal()));
            }
            current_node = current_node.children.as_mut().unwrap()[child_index].as_mut().unwrap();
        }

        let shift = 8 - accumulator_bit_count;
        let start = ((code as u32 << shift) & 0xff) as usize;
        let end = 1 << shift;
        
        let mut children = current_node.children.as_mut().unwrap();
        for j in start..(start + end) {
            // In Kotlin, .fill(terminal, start, end) copies the reference.
            // In Rust, we can't easily copy Box. We'll clone the terminal node.
            // However, the terminal node is small.
            children[j] = Some(Box::new(MutableNode::new_terminal(symbol, code_bit_count)));
        }
    }

    // Convert MutableNode tree to Arc<Node> tree
    fn convert(m_node: Box<MutableNode>) -> Arc<Node> {
        if let Some(m_children) = m_node.children {
            let mut children = m_children.into_iter()
                .map(|c| c.map(|cn| convert(cn)))
                .collect();
            Arc::new(Node {
                children: Some(children),
                symbol: m_node.symbol,
                terminal_bit_count: m_node.terminal_bit_count,
            })
        } else {
            Arc::new(Node {
                children: None,
                symbol: m_node.symbol,
                terminal_bit_count: m_node.terminal_bit_count,
            })
        }
    }

    convert(Box::new(root))
}

impl Huffman {
    pub fn encode<S: Write>(source: &[u8], sink: &mut S) -> io::Result<()> {
        let mut accumulator: i64 = 0;
        let mut accumulator_bit_count = 0;

        for &byte in source {
            let symbol = (byte & 0xff) as usize;
            let code = CODES[symbol];
            let code_bit_count = CODE_BIT_COUNTS[symbol] as i32;

            accumulator = (accumulator << code_bit_count) | (code as i64);
            accumulator_bit_count += code_bit_count as usize;

            while accumulator_bit_count >= 8 {
                accumulator_bit_count -= 8;
                sink.write_all(&[( (accumulator >> accumulator_bit_count) as u8 )])?;
            }
        }

        if accumulator_bit_count > 0 {
            accumulator = accumulator << (8 - accumulator_bit_count);
            accumulator = accumulator | ((0xffi64 >> accumulator_bit_count) & 0xff);
            sink.write_all(&[(accumulator as u8)])?;
        }
        Ok(())
    }

    pub fn encoded_length(bytes: &[u8]) -> usize {
        let mut bit_count: i64 = 0;

        for &byte in bytes {
            let byte_in = (byte & 0xff) as usize;
            bit_count += CODE_BIT_COUNTS[byte_in] as i64;
        }

        ((bit_count + 7) >> 3) as usize
    }

    pub fn decode<R: Read, S: Write>(source: &mut R, byte_count: i64, sink: &mut S) -> io::Result<()> {
        let mut node = Arc::clone(&ROOT);
        let mut accumulator: u32 = 0;
        let mut accumulator_bit_count = 0;

        for _ in 0..byte_count {
            let mut buf = [0u8; 1];
            source.read_exact(&mut buf)?;
            let byte_in = (buf[0] & 0xff) as u32;
            
            accumulator = (accumulator << 8) | byte_in;
            accumulator_bit_count += 8;

            while accumulator_bit_count >= 8 {
                let child_index = ((accumulator >> (accumulator_bit_count - 8)) & 0xff) as usize;
                
                let mut children = node.children.as_ref().expect("Non-terminal node must have children");
                node = Arc::clone(children[child_index].as_ref().expect("Huffman tree must be complete"));

                if node.children.is_none() {
                    // Terminal node.
                    sink.write_all(&[(node.symbol as u8)])?;
                    accumulator_bit_count -= node.terminal_bit_count as usize;
                    node = Arc::clone(&ROOT);
                } else {
                    // Non-terminal node.
                    accumulator_bit_count -= 8;
                }
            }
        }

        while accumulator_bit_count > 0 {
            let child_index = ((accumulator << (8 - accumulator_bit_count)) & 0xff) as usize;
            
            let mut children = node.children.as_ref().expect("Non-terminal node must have children");
            node = Arc::clone(children[child_index].as_ref().expect("Huffman tree must be complete"));

            if node.children.is_some() || node.terminal_bit_count > (accumulator_bit_count as i32) {
                break;
            }
            sink.write_all(&[(node.symbol as u8)])?;
            accumulator_bit_count -= node.terminal_bit_count as usize;
            node = Arc::clone(&ROOT);
        }
        Ok(())
    }
}