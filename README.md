# huffman_coding
Huffman Coding is a simple Rust-based implementation of Huffman encoding and decoding.

This project was inspired by a challenge from coding challenges.fyi and the algorithm implementation details were referenced from [huffman coding challenge](https://codingchallenges.fyi/challenges/challenge-huffman/) - Huffman Coding.
The implementation details are from [Huffman coding](https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/Huffman.html)
## The challenges I faced in this project are - 
1. **Handling Multiple References:** A major challenge was dealing with multiple references to the TreeNode and multiple owners in Rust. Traditional methods of using references didn't work due to Rust's ownership model, requiring the use of `Rc<HuffNode>` to create multiple references.
2. **Debugging BinaryHeap Comparison:** Debugging the comparison in BinaryHeap was challenging because it wasn't working correctly when comparing the weights and elements. This issue arose when both elements were None, leading to non-deterministic ordering due to the use of serde_json, which changes the ordering of HashMap. As a result, the heap provided the wrong order during decoding, affecting the correctness of the prefixes.
   ```rust
   // Old Implementation
   
   #[derive(Debug, Clone)]
    struct HuffNode {
      weight: u32,
      element: Option<char>,
      left: Option<TreeNodeRef>,
      right: Option<TreeNodeRef>
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
                (None, None) => Option::from(Ordering:Equal),
                (Some(_), None) => { Option::from(Ordering::Greater) }
                (None, Some(_)) => { Option::from(Ordering::Less) }
                (Some(el1), Some(el2)) => Option::from(el1.cmp(&el2))
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
                  (None, None) => {Ordering::Equal},
                  (Some(_), None) => { Ordering::Greater }
                  (None, Some(_)) => { Ordering::Less }
                  (Some(el1), Some(el2)) => el1.cmp(&el2)
             }
         } else {
            other.weight().cmp(&self.weight())
         }
       }
    }
   ```
   
    ```rust
   // new implementation
   #[derive(Debug, Clone)]
    struct HuffNode {
       weight: u32,
       element: Option<char>,
       left: Option<TreeNodeRef>,
       right: Option<TreeNodeRef>,
       id: u32
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
                    (Some(_), None) => { Option::from(Ordering::Greater) }
                    (None, Some(_)) => { Option::from(Ordering::Less) }
                    (Some(el1), Some(el2)) => Option::from(el1.cmp(&el2))
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
                    (Some(_), None) => { Ordering::Greater }
                    (None, Some(_)) => { Ordering::Less }
                    (Some(el1), Some(el2)) => el1.cmp(&el2)
                }
            } else {
                other.weight().cmp(&self.weight())
            }
        }
    }
   ```
   In the new implementation, an `id` field was added, generated incrementally, to ensure deterministic behavior when comparing nodes where the element is `None`.
3. **Encoding and Decoding:** Ensuring the use of compressed bits for file I/O was also challenging because Rust doesn't provide direct bit manipulation. Conversion from bytes `u8` to bits had to be implemented.
## Assumptions-
1. The implementation assumes that the unique characters in the input string are greater than or equal to 2.
2. It uses `serde_json` to store mappings, which may be less efficient compared to [Canonical Encoding](https://en.wikipedia.org/wiki/Canonical_Huffman_code), where mappings can be stored in $B*2^B$ bits of information (where B is the number of bits per symbol).
3. Custom error handling using the `thiserror` package was not implemented extensively, except for handling read file errors.

### Running the code
You can run the encoding algorithm using - 
```
cargo run -- /absolute-path-to-file
```

You can run the decoding algorithm using -
```
cargo run -- /absolute-path-to-huf-file -d
```

### Extra Dependencies
Additional dependencies used in this project:  
```toml
thiserror = "1.0.56" # custom error handling package
clap = { version = "4.5.0", features = ["derive"] } # command line argument parser packages
serde_json = "1.0.115" # serializing DataStructures to json, used for serializing hashmap to json
```


