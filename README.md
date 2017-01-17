# multi_mut

A bunch of extension methods on `HashMap` and `BTreeMap` that provide a safe API for getting multiple mutable references to values contained in them.
Runtime checks are done to prevent mutable aliasing.

### Disclaimer

**Note: As of the version 0.1.5, all transmutes of mutable references are removed. The funny business is now done with raw pointers.**

This crate performs some black magic behind the curtains (mutable transmutes). Whether this is safe to do or not depends on the Rust memory model, and
the particulars of that are not set in stone yet. **No mutable access is ever done through aliasing references and the uniqueness of the references is checked
before transmuting `&V` to `&mut V`.**

However the critical thing this crate depends on is: is it UB for `&V` and `&mut V` that point to the same value of type `V` to
*exist* momentarily? When trying to get a new mutable reference to a value inside `HashMap`, there may already exist a `&mut V` "in the wild", returned by the 
same getter method earlier. Inside an `unsafe` block, a `&V` is created and it's then checked against already existing references. If no earlier reference exists,
`&V` is transmuted to `&mut V`. This means that there is a moment where `&V` and `&mut V` may exist simultaneously.

Note that this happens inside an `unsafe` block, and there is some debate about to which degree the compiler should expect the type system invariants to hold
inside `unsafe` blocks (and even inside functions or modules that contain `unsafe` blocks.) On the other hand, there is also debate whether mere *existence* of mutable
aliasable references is UB, or is it accessing the value *through* them that is UB.
I'm not doing access through them, and I'm doing all the transmute trickery inside an `unsafe` block,
so I claim the black magic that happens in this crate to be on the conservative side. Nevertheless, be careful! This seems to work – and I don't see why it shouldn't –
as of Jan 2017, but as I said, the memory model is still evolving, and **I will take no responsibility of undefined behaviour possibly caused by this crate.**

## How to use

Add to Cargo.toml::

```
[dependencies]
multi_mut = "0.1"
```

Bring the extension trait to the scope in your code:
```
extern crate multi_mut;
use multi_mut::{HashMapMultiMut, BTreeMapMultiMut};
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

Quick & dirty list of available methods: (They work on both `HashMap` and `BTreeMap`, provided that you have the corresponding trait in scope. )
* `get_pair_mut(key, key)` Returns a pair of mutable references wrapped in `Option`
* `pair_mut(key, key)` Returns a pair of mutable references and panics if the keys don't exist.
* `get_triple_mut(key, key, key)`Returns a triple of mutable references wrapped in `Option`
* `triple_mut(key, key, key)` Returns a triple of mutable references and panics if the keys don't exist.
* `multi_mut(buffer)` and `iter_multi_mut(keys, buffer)` return arbitrary number of mutable references. Check out the example below.

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

