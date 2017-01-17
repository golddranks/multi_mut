# multi_mut

A bunch of extension methods on `HashMap` and `BTreeMap` that provide a safe API for getting multiple mutable references to values contained in them.
Runtime checks are done to prevent mutable aliasing.

## How to use

Add to Cargo.toml::

```
[dependencies]
multi_mut = "0.1"
```

Bring the extension trait to the scope in your code:
```
extern crate multi_mut;
use multi_mut::HashMapMultiMut;
```
or
```
extern crate multi_mut;
use multi_mut::BTreeMapMultiMut;
```

You can now have more than one mutable reference to your `HashMap` or `BTreeMap` safely!
```
    let (one, two) = map.get_pair_mut("key_one", "key_two").unwrap();
    
    assert_eq!(one, "value_one");
    assert_eq!(two, "value_two");

    one.push_str("_edited");
    two.push_str("_edited");

    assert_eq!(one, "value_one_edited");
    assert_eq!(two, "value_two_edited");
```

Quick & dirty list of available functions:
* `get_pair_mut(key, key)` Returns a pair of mutable references wrapped in `Option`
* `pair_mut(key, key)` Returns a pair of mutable references and panics if the keys don't exist.
* `get_triple_mut(key, key, key)`Returns a triple of mutable references wrapped in `Option`
* `triple_mut(key, key, key)` Returns a triple of mutable references and panics if the keys don't exist.
* `multi_mut()` and `iter_multi_mut()` return arbitrary number of mutable references. Check out the example below.

To prevent mutable aliasing, all functions will panic if the input keys aren't unique. None of the functions allocate.
`multi_mut()` and `iter_multi_mut()` perform a linear search over a buffer of pointers every time a mutable reference
is pulled out of the `HashMap`/`BTreeMap`. In practice, this is fast enough.

### How to use `multi_mut()` and `iter_multi_mut()`

`multi_mut()` and `iter_multi_mut()` need a mutable buffer to keep track of existing references to prevent mutable aliasing. 
See the line `let mut buffer = [std::ptr::null(); 3];` in the example. The size of the buffer determines how many values you can
pull out of the underlying `HashMap`/`BTreeMap`.

The difference between the two methods is that `multi_mut()` returns a wrapper which can be used to fetch mutable references
from `HashMap`/`BTreeMap` using the `get_mut(&K) -> Option<&mut V>` or `mut_ref(&K) -> &mut V` (this panics if the key doesn't exist) methods,
whereas `iter_multi_mut()` requires a list of keys up front, and then returns an iterator that spews out mutable references.

An example of `multi_mut()`:

```
    let mut buffer = [std::ptr::null(); 3];
    let mut wrapper = map.multi_mut(&mut buffer);
    
    let one = wrapper.get_mut("key_one").unwrap();
    let two = wrapper.get_mut("key_two").unwrap();
    let three = wrapper.get_mut("key_three").unwrap();

    assert_eq!(one, "value_one");
    assert_eq!(two, "value_two");
    assert_eq!(three, "value_three");

    one.push_str("_edited");
    two.push_str("_edited");
    three.push_str("_edited");

    assert_eq!(one, "value_one_edited");
    assert_eq!(two, "value_two_edited");
    assert_eq!(three, "value_three_edited");
```

An example of `iter_multi_mut()`:

```
    let mut buffer = [std::ptr::null(); 3];
    let keys = ["key_one", "key_two", "key_three"];
    let mut wrapper = map.iter_multi_mut(&keys, &mut buffer);
    
    let one = wrapper.next().unwrap();
    let two = wrapper.next().unwrap();
    let three = wrapper.next().unwrap();

    assert_eq!(one, "value_one");
    assert_eq!(two, "value_two");
    assert_eq!(three, "value_three");

    one.push_str("_edited");
    two.push_str("_edited");
    three.push_str("_edited");

    assert_eq!(one, "value_one_edited");
    assert_eq!(two, "value_two_edited");
    assert_eq!(three, "value_three_edited");
```

