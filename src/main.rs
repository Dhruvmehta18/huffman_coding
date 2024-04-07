use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::Parser;
use log::error;
use serde_json::Value;
use thiserror::Error;

#[derive(Parser, Default, Debug)]
#[command(
    version,
    about,
    long_about = "huffman compression implementation in rust"
)]
#[clap(name = "compressor")]
struct Args {
    #[arg(help = "path of file to compress", required = true)]
    path: String,
    #[arg(short, help = "option to decode huffman encoded string")]
    decode: bool,
}

#[derive(Error, Debug)]
enum FindError {
    #[error("Error reading File: {0}")]
    ReadFileError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
struct HuffNode {
    weight: u32,
    element: Option<char>,
    left: Option<TreeNodeRef>,
    right: Option<TreeNodeRef>,
    id: u32,
}

type TreeNodeRef = Rc<RefCell<HuffNode>>;

impl Display for HuffNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HuffNode {{ weight: {}, element: {:?} id: {:?} }}",
            self.weight, self.element, self.id
        )
    }
}

impl HuffNode {
    fn new(left: HuffNode, right: HuffNode, id: u32) -> HuffNode {
        Self {
            weight: left.weight() + right.weight(),
            element: None,
            left: Option::from(Rc::new(RefCell::new(left))),
            right: Option::from(Rc::new(RefCell::new(right))),
            id,
        }
    }

    fn is_leaf(&self) -> bool {
        match self.element {
            None => false,
            Some(_) => true,
        }
    }

    fn weight(&self) -> u32 {
        self.weight
    }
}

impl PartialEq for HuffNode {
    fn eq(&self, other: &Self) -> bool {
        if other.weight().eq(&self.weight()) {
            self.element.eq(&other.element)
        } else {
            other.weight().eq(&self.weight())
        }
    }
}

impl PartialOrd for HuffNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other.weight().eq(&self.weight()) {
            match (self.element, other.element) {
                (None, None) => Some(other.id.cmp(&self.id)),
                (Some(_), None) => Option::from(Ordering::Greater),
                (None, Some(_)) => Option::from(Ordering::Less),
                (Some(el1), Some(el2)) => Option::from(el1.cmp(&el2)),
            }
        } else {
            Some(other.weight().cmp(&self.weight()))
        }
    }
}

impl Ord for HuffNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.weight().eq(&self.weight()) {
            match (self.element, other.element) {
                (None, None) => other.id.cmp(&self.id),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some(el1), Some(el2)) => el1.cmp(&el2),
            }
        } else {
            other.weight().cmp(&self.weight())
        }
    }
}

impl Eq for HuffNode {}

struct BitsEncoder {
    bytes: Vec<u8>,
    current_byte: u8,
    byte_count: usize,
    bits_count: u64,
}

const BITS_PER_BYTE: usize = 8;

impl BitsEncoder {
    fn new() -> Self {
        Self {
            bytes: vec![],
            current_byte: 0,
            byte_count: 0,
            bits_count: 0,
        }
    }

    fn add_bit(&mut self, bit: bool) {
        if self.byte_count == BITS_PER_BYTE {
            self.flush_current_byte();
        }
        if bit {
            self.current_byte |= 1 << (BITS_PER_BYTE - self.byte_count - 1);
        }
        self.byte_count += 1;
        self.bits_count += 1;
    }

    fn flush_current_byte(&mut self) {
        self.bytes.push(self.current_byte);
        self.current_byte = 0;
        self.byte_count = 0;
    }

    fn encode(&self) -> &[u8] {
        &self.bytes
    }
}

struct HuffmanDecoder {
    bytes: Vec<u8>,
    path: PathBuf,
}

impl HuffmanDecoder {
    fn new(bytes: Vec<u8>, path: PathBuf) -> Self {
        Self { bytes, path }
    }

    fn get_mappings(&self) -> (HashMap<char, u32>, usize, u64) {
        // Read from the file line by line
        let mut counter = 0;
        let mut header_byte_counter = 0;
        for byte in self.bytes.bytes() {
            let b = byte.unwrap();
            // print!("{:?}", b);
            header_byte_counter += 1;
            if b == b'\n' {
                break;
            }
            counter += 1;
        }
        println!("number bytes {:?}", &self.bytes[0..counter]);
        let file_size = bytes_to_u64(&self.bytes[0..counter]);

        let mut buf = vec![];
        let mut counter_n = 0;
        println!("len ========={}", self.bytes.len());
        for byte in self.bytes[counter + 1..].bytes() {
            let b = byte.unwrap();
            header_byte_counter += 1;
            if b == b'\n' {
                counter_n += 1;
                buf.push(b);
                if counter_n == 2 {
                    break;
                };
            } else {
                counter_n = 0;
                buf.push(b);
            }
        }
        println!("{:?}", &buf);
        // let huff_mappings = String::from_utf8_lossy(&buf);
        let mappings: Value = serde_json::from_slice(&buf).unwrap();
        println!("{:?}", &mappings);
        println!("{:?}", &header_byte_counter);

        let huff_map: HashMap<char, u32> = mappings
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.chars().next().unwrap(), v.as_u64().unwrap() as u32))
            .collect();
        (huff_map, header_byte_counter, file_size)
    }

    fn decode(&self) {
        let (mappings, header_byte_counter, file_size) = self.get_mappings();
        println!("{:?}", &mappings);
        let mut priority_queue = get_priority_queue(&mappings);

        match get_huffman_tree_node(&mut priority_queue) {
            None => {
                panic!("Something went wrong")
            }
            Some(node) => {
                println!("root node {}", node);
                let huff_map = traverse_and_get_prefixes(Rc::new(RefCell::new(node.clone())));

                for (key, value) in &huff_map {
                    let bit_str: String =
                        value.iter().map(|x| if *x { '1' } else { '0' }).collect();
                    println!("{} | {}", key, bit_str);
                }
                self.decoding(&Rc::new(RefCell::new(node)), header_byte_counter, file_size);
            }
        }
    }

    fn decoding(&self, huff_node: &TreeNodeRef, start_from: usize, file_size: u64) {
        println!("{},{}", start_from, file_size);
        let mut tmp_node = Rc::clone(&huff_node);
        let mut buffer = Vec::new();
        let mut counter: u64 = 0;
        for byte in &self.bytes[start_from..] {
            // print!("{:?}, ", byte);
            for i in (0..BITS_PER_BYTE).rev() {
                if counter >= file_size {
                    break;
                }
                counter += 1;
                let bit = (*byte >> i) & 1;
                if bit == 0 {
                    // print!("0");
                    let tmp = Rc::clone(&tmp_node);
                    match &tmp.borrow().left {
                        None => {
                            panic!("File is invalid");
                        }
                        Some(node_ref) => {
                            let node = Rc::clone(node_ref);
                            let next_node = match node.borrow().element {
                                None => Rc::clone(&node),
                                Some(c) => {
                                    buffer.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                                    Rc::clone(huff_node)
                                }
                            };
                            tmp_node = next_node;
                        }
                    };
                } else {
                    // print!("1");
                    let tmp = Rc::clone(&tmp_node);
                    match &tmp.borrow().right {
                        None => {
                            panic!("File is invalid");
                        }
                        Some(node_ref) => {
                            let node = Rc::clone(node_ref);
                            let next_node = match node.borrow().element {
                                None => Rc::clone(&node),
                                Some(c) => {
                                    buffer.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                                    Rc::clone(huff_node)
                                }
                            };
                            tmp_node = next_node;
                        }
                    };
                }
            }
        }

        match fs::write(&self.path, buffer) {
            Ok(_) => {
                println!("File written successfully")
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        };
    }
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut result: u64 = 0;
    for byte in bytes {
        if byte.is_ascii_digit() {
            result = result * 10 + (*byte as char).to_digit(10).unwrap() as u64;
        } else {
            panic!("cannot convert non numeric to number")
        }
    }
    result
}

fn main() {
    let args = Args::parse();

    let path = args.path;
    let dec = args.decode;

    if dec {
        decode(&path);
    } else {
        encode(&path);
    }
}

fn encode(path: &String) {
    let str_result = fs::read_to_string(&path).map_err(|err| FindError::ReadFileError(err));

    match str_result {
        Ok(file_str) => {
            let huff_freq = get_frequency_from_string(&file_str);
            println!("{:?}", &huff_freq);

            if huff_freq.len() < 2 {
                panic!("Cannot build huffman for less than 2 unique character");
            }

            let mut priority_queue = get_priority_queue(&huff_freq);

            if let Some(node) = get_huffman_tree_node(&mut priority_queue) {
                println!("root node {}", node);
                let huff_map = traverse_and_get_prefixes(Rc::new(RefCell::new(node)));

                for (key, value) in &huff_map {
                    let bit_str: String =
                        value.iter().map(|x| if *x { '1' } else { '0' }).collect();
                    println!("{} | {}", key, bit_str);
                }

                let mut bits_encoder = BitsEncoder::new();

                for c in file_str.chars() {
                    for bit in huff_map.get(&c).unwrap() {
                        // print!("{}", if *bit { "1"} else {"0"});
                        bits_encoder.add_bit(*bit)
                    }
                }
                bits_encoder.flush_current_byte();
                let path_buf = Path::new(&path);
                if let Some(path) = path_buf.parent() {
                    let compress_file_path = path
                        .join(path_buf.file_stem().unwrap().to_str().unwrap().to_owned() + ".huf");
                    let mappings = serialize_huffman_mappings(&huff_freq).unwrap();
                    let mapping_bytes = (mappings + "\n\n").into_bytes();
                    println!("le === ==== {}", mapping_bytes.len());
                    match fs::write(
                        &compress_file_path,
                        bits_encoder.bits_count.to_string().to_owned() + "\n",
                    ) {
                        Ok(_) => {
                            println!("size {} written to file", &bits_encoder.bits_count);
                        }
                        Err(err) => {
                            panic!("writing failed {}", err);
                        }
                    }

                    let mut file = OpenOptions::new()
                        .append(true)
                        .open(&compress_file_path)
                        .unwrap();

                    match file.write_all(&mapping_bytes) {
                        Ok(_) => {
                            println!("mapping written to file");
                        }
                        Err(e) => {
                            panic!("writing failed {}", e);
                        }
                    }

                    match file.write_all(&bits_encoder.encode()) {
                        Ok(_) => {
                            println!("File written successfully")
                        }
                        Err(err) => {
                            eprintln!("{err}")
                        }
                    }
                };
            } else {
                panic!("something went wrong");
            }
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

fn get_priority_queue(huff_freq: &HashMap<char, u32>) -> BinaryHeap<HuffNode> {
    let mut counter = 0;
    let mut priority_queue = BinaryHeap::new();
    for (key, value) in huff_freq {
        println!("Key: {}, Value: {}", key, value);
        priority_queue.push(HuffNode {
            weight: *value,
            element: Some(*key),
            left: None,
            right: None,
            id: counter,
        });
        counter += 1;
    }

    priority_queue
}

fn get_huffman_tree_node(priority_queue: &mut BinaryHeap<HuffNode>) -> Option<HuffNode> {
    let mut counter = priority_queue.len() as u32;
    while priority_queue.len() > 1 {
        let tmp1 = priority_queue.pop().unwrap();
        let tmp2 = priority_queue.pop().unwrap();
        println!("tmp1 = {} ,,, tmp2 = {}", tmp1, tmp2);
        let tree_node = Option::from(Rc::new(RefCell::new(HuffNode::new(tmp1, tmp2, counter))));
        priority_queue.push(tree_node.unwrap().borrow().to_owned());
        counter += 1;
    }

    priority_queue.pop()
}

fn decode(path: &String) {
    let path = Path::new(&path);
    println!("file name {:?}", path.file_name().unwrap());
    println!("file name {:?}", path.extension().unwrap());
    match &path.parent() {
        None => {
            println!("Parent folder not found");
        }
        Some(parent_path) => {
            let file_write_path = parent_path
                .join(path.file_stem().unwrap().to_str().unwrap().to_owned() + "_decode" + ".txt");
            let file = File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            let mut buf_vec = Vec::new();
            reader
                .read_to_end(&mut buf_vec)
                .expect("Error reading file");
            println!("{}", buf_vec.len());
            let huffman_decoder = HuffmanDecoder::new(buf_vec, file_write_path);
            huffman_decoder.decode()
        }
    }
}

fn serialize_huffman_mappings(map: &HashMap<char, u32>) -> serde_json::error::Result<String> {
    serde_json::to_string(map)
}

fn traverse_and_get_prefixes(node: TreeNodeRef) -> HashMap<char, Vec<bool>> {
    let mut prefix_map = HashMap::new();
    traverse_and_get_prefixes_int(&Some(node), &mut Vec::new(), &mut prefix_map);
    prefix_map
}

fn traverse_and_get_prefixes_int(
    node: &Option<TreeNodeRef>,
    bits: &mut Vec<bool>,
    map: &mut HashMap<char, Vec<bool>>,
) {
    if let Some(ref node_ref) = node {
        let node_bor = node_ref.borrow();
        if node_bor.is_leaf() {
            // let str: String = bits.to_vec().iter().map(|x| if *x { '1' } else { '0' }).collect();
            // println!("key: {}, value: {}", node_bor.element.unwrap(), str);
            map.insert(node_bor.element.unwrap(), bits.to_vec());
            return;
        } else {
            bits.push(false);
            traverse_and_get_prefixes_int(&node_bor.left, bits, map);
            bits.pop();
            bits.push(true);
            traverse_and_get_prefixes_int(&node_bor.right, bits, map);
            bits.pop();
        }
    } else {
        return;
    }
}

fn get_frequency_from_string(s: &String) -> HashMap<char, u32> {
    let mut huff_map = HashMap::new();

    for character in s.chars() {
        *huff_map.entry(character).or_insert(0) += 1
    }

    huff_map
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::env;

    const PATH_TO_FILE: &str = "huffman.txt";
    const PATH_TO_DECODE: &str = "huffman.huf";

    const PATH_DECODED_FILE: &str = "huffman_decode.txt";

    fn files_have_same_content(file1_path: &str, file2_path: &str) -> bool {
        // Read the contents of both files
        let file1_content = match fs::read_to_string(file1_path) {
            Ok(content) => content,
            Err(_) => return false, // Return false if unable to read file1
        };

        let file2_content = match fs::read_to_string(file2_path) {
            Ok(content) => content,
            Err(_) => return false, // Return false if unable to read file2
        };
        // Compare the contents of both files
        file1_content == file2_content
    }

    #[test]
    fn check_frequency_of_some_english_characters() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Combine the current directory with the relative path
        let file_path = current_dir.join(PATH_TO_FILE);
        let file_str = fs::read_to_string(file_path).expect("It should be valid path");

        let hash_map = get_frequency_from_string(&file_str);

        assert_eq!(hash_map.get(&'X'), Some(333).as_ref());
        assert_eq!(hash_map.get(&'t'), Some(223000).as_ref())
    }

    #[test]
    fn check_frequency_of_some_non_english_characters() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Combine the current directory with the relative path
        let file_path = current_dir.join(PATH_TO_FILE);
        let file_str = fs::read_to_string(file_path).expect("It should be valid path");

        let hash_map = get_frequency_from_string(&file_str);

        assert_eq!(hash_map.get(&'â'), Some(56).as_ref());
        assert_eq!(hash_map.get(&'À'), Some(5).as_ref());
    }

    #[test]
    fn check_frequency_of_some_other_characters() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Combine the current directory with the relative path
        let file_path = current_dir.join(PATH_TO_FILE);
        let file_str = fs::read_to_string(file_path).expect("It should be valid path");

        let hash_map = get_frequency_from_string(&file_str);

        assert_eq!(hash_map.get(&'\n'), Some(73589).as_ref());
        assert_eq!(hash_map.get(&'$'), Some(2).as_ref());
    }

    #[test]
    fn encode_and_decode_should_generate_same_file_small() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Combine the current directory with the relative path
        let file_path = current_dir.join("small.txt");

        encode(&file_path.to_str().unwrap().to_string());
        let file_decode_path = current_dir.join("small.huf");
        decode(&file_decode_path.to_str().unwrap().to_string());
        let file_decoded_path = current_dir.join("small_decode.txt");
        assert_eq!(
            files_have_same_content(
                file_path.to_str().unwrap(),
                file_decoded_path.to_str().unwrap()
            ),
            true
        );
    }

    #[test]
    fn encode_and_decode_should_generate_same_file() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Combine the current directory with the relative path
        let file_path = current_dir.join(PATH_TO_FILE);

        encode(&file_path.to_str().unwrap().to_string());
        let file_decode_path = current_dir.join(PATH_TO_DECODE);
        decode(&file_decode_path.to_str().unwrap().to_string());
        let file_decoded_path = current_dir.join(PATH_DECODED_FILE);
        assert_eq!(
            files_have_same_content(
                file_path.to_str().unwrap(),
                file_decoded_path.to_str().unwrap()
            ),
            true
        );
    }
}
