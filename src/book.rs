use shakmaty::{
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode,
};
use shakmaty_uci::UciMove;
use std::io::{BufReader, Read};
use std::str::FromStr;
use std::{collections::HashMap, io::Cursor};

#[derive(Debug)]
pub struct BookMove {
    uci_move: UciMove,
    weight: u16,
}

pub struct Book {
    positions: HashMap<Zobrist64, Vec<BookMove>>,
}

impl Book {
    pub fn load(path: &str) -> Self {
        let file = std::fs::File::open(path).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("Failed to read file");
        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut positions = HashMap::new();
        let mut cursor = Cursor::new(data);
        let mut chunk = [0u8; 16];

        loop {
            match cursor.read_exact(&mut chunk) {
                Ok(()) => {
                    // Read 8 bytes to get zobrist hash
                    let hash = Zobrist64::from(u64::from_be_bytes(chunk[0..8].try_into().unwrap()));

                    // Parse move
                    let uci_move = Self::bytes_to_uci_move(chunk[8], chunk[9]);

                    // Parse weight
                    let weight = u16::from_be_bytes(chunk[10..12].try_into().unwrap());

                    // Insert
                    positions
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(BookMove { uci_move, weight });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => {
                    panic!("Error reading bytes: {}", e);
                }
            }
        }

        Book { positions }
    }

    pub fn moves(&self, pos: &Chess) -> Vec<UciMove> {
        let hash = pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
        match self.positions.get(&hash) {
            None => vec![],
            Some(moves) => moves.iter().map(|m| m.uci_move.clone()).collect::<Vec<_>>(),
        }
    }

    fn bytes_to_uci_move(b1: u8, b2: u8) -> UciMove {
        // PolyGlot uses big-endian for the 2-byte move field
        let combined = ((b1 as u16) << 8) | (b2 as u16);

        // Decode according to PolyGlot spec:
        let to_square = (combined & 0x3F) as u8;
        let from_square = ((combined >> 6) & 0x3F) as u8;
        let promotion = ((combined >> 12) & 0xF) as u8;

        // Split into file/rank for readability
        let from_file = from_square % 8;
        let from_rank = from_square / 8;
        let to_file = to_square % 8;
        let to_rank = to_square / 8;

        let file_char = |f| (b'a' + f) as char;
        let rank_char = |r| (b'1' + r) as char;

        let mut uci = format!(
            "{}{}{}{}",
            file_char(from_file),
            rank_char(from_rank),
            file_char(to_file),
            rank_char(to_rank)
        );

        // PolyGlot promotion codes: 1=n, 2=b, 3=r, 4=q
        match promotion {
            1 => uci.push('n'),
            2 => uci.push('b'),
            3 => uci.push('r'),
            4 => uci.push('q'),
            _ => (), // no promotion
        };

        UciMove::from_str(&uci).unwrap()
    }
}
