### Overview

Broccoli is a broadphase collision detection library. 
The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).

### Inner projects

The broccoli_demo inner project is meant to show case the use of these algorithms. 
The report inner project generates benches used in the [broccoli book](https://tiby312.github.io/broccoli_report).

### Screenshot

Screen capture from the inner dinotree_alg_demo project.

<img src="./assets/screenshot.gif" alt="screenshot">

### Example

```rust
use broccoli::prelude::*;

fn main() {
    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), 0),
        bbox(rect(15, 20, 15, 20), 0),
        bbox(rect(5, 15, 5, 15), 0),
    ];

    //Create a layer of direction.
    let mut ref_aabbs = aabbs.iter_mut().collect::<Vec<_>>();

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::new(&mut ref_aabbs);

    //Find all colliding aabbs.
    tree.find_colliding_pairs_mut(|a, b| {
        *a += 1;
        *b += 1;
    });

    assert_eq!(aabbs[0].inner, 1);
    assert_eq!(aabbs[1].inner, 0);
    assert_eq!(aabbs[2].inner, 1);
}

```
