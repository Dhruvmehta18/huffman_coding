# huffman_coding
Simple Rust based implementation of huffman encoding and decoding.

This project is from the challenge from this website https://codingchallenges.fyi/challenges/challenge-huffman/
For algorithm implementation details I have taken reference from this https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/Huffman.html


## The challenges I faced in this project are - 
1. I faced major challenge while making multiple references to the TreeNode and multiple owners in rust. As rust follow ownership model using traditional methods of references doesn't work, and we have to use `Rc::RefCell<HuffNode>` type of node to make multiple references
2. I faced major debugging challenge with BinaryHeap comparison as it was not working correctly when I compare first the weights and elements because when the both element was `None` then ordering was not deterministic as when using serde_json the ordering of Hashmap is changed. So when decoding the heap will gives wrong order which will make prefixes wrong.
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
   In new implementation i also have added id which is generated incrementally, so I can compare two nodes where the element is `None` which makes the BinaryHeap pop deterministic.
3. When encoding and decoding i also have to make sure that I use compressed bits to the file and rust doesn't provide direct implementation of bits, so will have to convert bytes `u8` to bits

## Assumptions-
1. I have taken assumption that the unique characters in string should be greater than equal to 2
2. I have also use `serde_json` to store mappings which will be less efficient compare to [Canonical Encoding](https://en.wikipedia.org/wiki/Canonical_Huffman_code) in which the mappings can be stored in $B*2^B$ bits of information (where B is the number of bits per symbol).
3. I have also `panic`(rust term) in most cases and have not done custom error handling which can be done using `thiserror` package (have done it for read file error though).

### Running the code
You can run the encoding algorithm using - 
```
cargo run -- /absolute-path-to-file
```

You can run the decoding algorithm using -
```
cargo run -- /absolute-path-to-huf-file -d
```

Extra Dependencies which I have used are  
```toml
thiserror = "1.0.56" # custom error handling package
clap = { version = "4.5.0", features = ["derive"] } # command line argument parser packages
serde_json = "1.0.115" # serializing DataStructures to json, used for serializing hashmap to json
```


