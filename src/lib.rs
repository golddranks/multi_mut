use std::borrow::Borrow;
use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashMap;
use std::mem::transmute;
use std::slice::Iter;

fn ref_eq<'a, 'b, T>(thing: &'a T, other: &'b T) -> bool {
    (thing as *const T) == (other as *const T)
}

/// Endows HashMap with extension methods that help getting multiple mutable references to the values contained in it.
/// Runtime-checking is done to ensure that this is safe: the returned mutable references are guaranteed not to alias.
pub trait HashMapMultiMut {
    type Value;
    type Key: Hash + Eq;

    fn get_pair_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q) -> Option<(&mut Self::Value, &mut Self::Value)>
        where Self::Key: Borrow<Q>, Q: Hash + Eq;

    fn pair_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q) -> (&mut Self::Value, &mut Self::Value)
        where Self::Key: Borrow<Q>, Q: Hash + Eq;

    fn get_triple_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q, k_3: &Q) -> Option<(&mut Self::Value, &mut Self::Value, &mut Self::Value)>
        where Self::Key: Borrow<Q>, Q: Hash + Eq;

    fn triple_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q, k_3: &Q) -> (&mut Self::Value, &mut Self::Value, &mut Self::Value)
        where Self::Key: Borrow<Q>, Q: Hash + Eq;

    fn multi_mut<'a>(&'a mut self, buffer: &'a mut [*const Self::Value]) -> HashMapMutWrapper<Self::Key, Self::Value>;

    fn iter_multi_mut<'a, Q: ?Sized>(&'a mut self, k: &'a [&'a Q], buffer: &'a mut [*const Self::Value]) -> MultiMutIter<Q, Self::Key, Self::Value>
        where Self::Key: Borrow<Q>, Q: Hash + Eq;
}


impl<K: Hash + Eq, V> HashMapMultiMut for HashMap<K, V> {
    type Value = V;
    type Key = K;

    #[allow(mutable_transmutes)]
    fn get_pair_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q) -> Option<(&mut V, &mut V)>
        where K: Borrow<Q>, Q: Hash + Eq
    {

        let v_1 = self.get(k_1);
        let v_2 = self.get(k_2);

        match (v_1, v_2) {
            (Some(v_1), Some(v_2)) => {
                if ref_eq(v_1, v_2) {
                    None
                } else {
                    unsafe { Some((transmute(v_1), transmute(v_2))) }   // This is safe to do because we checked that v_1 and v_2 don't alias,
                                                                        // and this function consumed a &mut self, which locks the HashMap so that
                                                                        // no further aliasing references will be created during the lifetime of these
                                                                        // references.
                }
            },
            _ => None,
        }
    }

    #[allow(mutable_transmutes)]
    fn pair_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q) -> (&mut V, &mut V)
        where K: Borrow<Q>, Q: Hash + Eq
    {

        let v_1 = &self[k_1];
        let v_2 = &self[k_2];
        if ref_eq(v_1, v_2) {
            panic!("The keys pointed to the same value! Only non-overlapping values can be handled.")
        } else {
            unsafe { (transmute(v_1), transmute(v_2)) } // This is safe to do because we checked that v_1 and v_2 don't alias,
                                                        // and this function consumed a &mut self, which locks the HashMap so that
                                                        // no further aliasing references will be created during the lifetime of these
                                                        // references.
        }
    }

    #[allow(mutable_transmutes)]
    fn get_triple_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q, k_3: &Q) -> Option<(&mut V, &mut V, &mut V)>
        where K: Borrow<Q>, Q: Hash + Eq
    {

        let v_1 = self.get(k_1);
        let v_2 = self.get(k_2);
        let v_3 = self.get(k_3);

        match (v_1, v_2, v_3) {
            (Some(v_1), Some(v_2), Some(v_3)) => {
                if ref_eq(v_1, v_2) || ref_eq(v_2, v_3) || ref_eq(v_1, v_3) {
                    None
                } else {
                    unsafe { Some((transmute(v_1), transmute(v_2), transmute(v_3))) } 
                        // This is safe to do because we checked that v_1, v_2 and v_3 don't alias,
                        // and this function consumed a &mut self, which locks the HashMap so that
                        // no further aliasing references will be created during the lifetime of these
                        // references.
                }
            },
            _ => None,
        }
    }

    #[allow(mutable_transmutes)]
    fn triple_mut<Q: ?Sized>(&mut self, k_1: &Q, k_2: &Q, k_3: &Q) -> (&mut V, &mut V, &mut V)
        where K: Borrow<Q>, Q: Hash + Eq
    {

        let v_1 = &self[k_1];
        let v_2 = &self[k_2];
        let v_3 = &self[k_3];
        if ref_eq(v_1, v_2) || ref_eq(v_2, v_3) || ref_eq(v_1, v_3) {
            panic!("The keys pointed to the same value! Only non-overlapping values can be handled.")
        } else {
            unsafe { (transmute(v_1), transmute(v_2), transmute(v_3)) }
                // This is safe to do because we checked that v_1, v_2 and v_3 don't alias,
                // and this function consumed a &mut self, which locks the HashMap so that
                // no further aliasing references will be created during the lifetime of these
                // references.
        }
    }

    fn multi_mut<'a>(&'a mut self, buffer: &'a mut [*const V]) -> HashMapMutWrapper<K, V>
    {
        HashMapMutWrapper { used: 0, map: self, buffer: buffer }
    }

    fn iter_multi_mut<'a, Q: ?Sized>(&'a mut self, keys: &'a [&'a Q], buffer: &'a mut [*const V]) -> MultiMutIter<Q, K, V>
        where K: Borrow<Q>, Q: Hash + Eq
    {
        MultiMutIter { mut_wrapper: self.multi_mut(buffer), keys: keys.into_iter() }
    }

}

pub struct HashMapMutWrapper<'a, K: 'a, V: 'a>
        where K: Hash + Eq
{
    used: usize,
    map: &'a mut HashMap<K, V>,
    buffer: &'a mut [*const V],
}

impl<'a, K, V> HashMapMutWrapper<'a, K, V>
        where K: Hash + Eq
{

    #[allow(mutable_transmutes)]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&'a mut V>
        where K: Borrow<Q>, Q: Hash + Eq
    {
        if self.used == self.buffer.len() {
            panic!("Buffer space is depleted!");
        }
        let v = if let Some(v) = self.map.get(k) { v } else { return None };    // Note: should we be worried about aliased reads happening in get()?
                                                                                // after all, there might exist a &mut ref to the value at this point.
                                                                                // However, get() doesn't probably read through &V, it accesses only &K.
        let ptr = v as *const V;
        for old_ptr in &self.buffer[0..self.used] {
            if ptr == *old_ptr {
                panic!("No aliased references allowed! This key has been already used.");
            }
        }
        self.buffer[self.used] = ptr;
        self.used += 1;

        Some(unsafe{ transmute(v) })
    }

    pub fn mut_ref<Q: ?Sized>(&mut self, k: &Q) -> &'a mut V
        where K: Borrow<Q>, Q: Hash + Eq {
            match self.get_mut(k) {
                Some(v) => v,
                None => panic!("No such key!"),
            }
        }
}

pub struct MultiMutIter<'a, Q: ?Sized + 'a, K: 'a, V: 'a>
        where K: Borrow<Q> + Hash + Eq, Q: Hash + Eq
{
    mut_wrapper: HashMapMutWrapper<'a, K, V>,
    keys: Iter<'a, &'a Q>,
}

impl<'a, Q: ?Sized, K, V> Iterator for MultiMutIter<'a, Q, K, V>
        where K: Borrow<Q> + Hash + Eq, Q: Hash + Eq
{
    type Item = &'a mut V;

    #[allow(mutable_transmutes)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.mut_wrapper.used == self.mut_wrapper.buffer.len() { return None };
        match self.keys.next() {
            Some(q) => {
                self.mut_wrapper.get_mut(q)
            },
            None => None,
        }
        
    } 
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use HashMapMultiMut;

    fn populate_hashmap() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("key_one".into(), "value_one".into());
        map.insert("key_two".into(), "value_two".into());
        map.insert("key_three".into(), "value_three".into());
        map.insert("key_four".into(), "value_four".into());
        map.insert("key_five".into(), "value_five".into());
        map.insert("key_six".into(), "value_six".into());
        map
    }

    #[test]
    fn test_pair_success() {
        let mut map = populate_hashmap();
        let (one, two): (&mut String, &mut String) = map.get_pair_mut("key_one", "key_two").unwrap();
        
        assert_eq!(one, "value_one");
        assert_eq!(two, "value_two");

        one.push_str("_edited");
        two.push_str("_edited");

        assert_eq!(one, "value_one_edited");
        assert_eq!(two, "value_two_edited");
    }

    #[test]
    fn test_pair_nonexistent_key() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_pair_mut("key_one", "key_hundred"), None);
    }

    #[test]
    fn test_pair_overlap() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_pair_mut("key_one", "key_one"), None);
    }

    #[test]
    fn test_pair_panic_success() {
        let mut map = populate_hashmap();
        let (one, two): (&mut String, &mut String) = map.pair_mut("key_one", "key_two");
        
        assert_eq!(one, "value_one");
        assert_eq!(two, "value_two");

        one.push_str("_edited");
        two.push_str("_edited");

        assert_eq!(one, "value_one_edited");
        assert_eq!(two, "value_two_edited");
    }

    #[test]
    #[should_panic]
    fn test_pair_panic_nonexistent_key() {
        let mut map = populate_hashmap();
        map.pair_mut("key_one", "key_hundred");
    }

    #[test]
    #[should_panic]
    fn test_pair_panic_overlap() {
        let mut map = populate_hashmap();
        map.pair_mut("key_one", "key_one");
    }

    #[test]
    fn test_triple_success() {
        let mut map = populate_hashmap();
        let (one, two, three): (&mut String, &mut String, &mut String) = map.get_triple_mut("key_one", "key_two", "key_three").unwrap();
        
        assert_eq!(one, "value_one");
        assert_eq!(two, "value_two");
        assert_eq!(three, "value_three");

        one.push_str("_edited");
        two.push_str("_edited");
        three.push_str("_edited");

        assert_eq!(one, "value_one_edited");
        assert_eq!(two, "value_two_edited");
        assert_eq!(three, "value_three_edited");
    }

    #[test]
    fn test_triple_nonexistent_key() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_triple_mut("key_one", "key_hundred", "key_three"), None);
    }

    #[test]
    fn test_triple_overlap_1() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_triple_mut("key_one", "key_two", "key_one"), None);
    }

    #[test]
    fn test_triple_overlap_2() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_triple_mut("key_two", "key_two", "key_three"), None);
    }

    #[test]
    fn test_triple_overlap_3() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_triple_mut("key_one", "key_three", "key_three"), None);
    }

    #[test]
    fn test_triple_overlap_4() {
        let mut map = populate_hashmap();
        assert_eq!(map.get_triple_mut("key_one", "key_one", "key_one"), None);
    }

    #[test]
    fn test_triple_panic_success() {
        let mut map = populate_hashmap();
        let (one, two, three): (&mut String, &mut String, &mut String) = map.triple_mut("key_one", "key_two", "key_three");
        
        assert_eq!(one, "value_one");
        assert_eq!(two, "value_two");
        assert_eq!(three, "value_three");

        one.push_str("_edited");
        two.push_str("_edited");
        three.push_str("_edited");

        assert_eq!(one, "value_one_edited");
        assert_eq!(two, "value_two_edited");
        assert_eq!(three, "value_three_edited");
    }

    #[test]
    #[should_panic]
    fn test_triple_panic_nonexistent_key() {
        let mut map = populate_hashmap();
        map.triple_mut("key_one", "key_hundred", "key_three");
    }

    #[test]
    #[should_panic]
    fn test_triple_panic_overlap_1() {
        let mut map = populate_hashmap();
        map.triple_mut("key_one", "key_two", "key_one");
    }

    #[test]
    #[should_panic]
    fn test_triple_panic_overlap_2() {
        let mut map = populate_hashmap();
        map.triple_mut("key_two", "key_two", "key_three");
    }

    #[test]
    #[should_panic]
    fn test_triple_panic_overlap_3() {
        let mut map = populate_hashmap();
        map.triple_mut("key_one", "key_three", "key_three");
    }

    #[test]
    #[should_panic]
    fn test_triple_panic_overlap_4() {
        let mut map = populate_hashmap();
        map.triple_mut("key_one", "key_one", "key_one");
    }

    #[test]
    fn test_multi_success() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
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
    }

    #[test]
    fn test_multi_ref_success() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let mut wrapper = map.multi_mut(&mut buffer);
        
        let one = wrapper.mut_ref("key_one");
        let two = wrapper.mut_ref("key_two");
        let three = wrapper.mut_ref("key_three");

        assert_eq!(one, "value_one");
        assert_eq!(two, "value_two");
        assert_eq!(three, "value_three");

        one.push_str("_edited");
        two.push_str("_edited");
        three.push_str("_edited");

        assert_eq!(one, "value_one_edited");
        assert_eq!(two, "value_two_edited");
        assert_eq!(three, "value_three_edited");
    }

    #[test]
    #[should_panic]
    fn test_multi_over_capacity() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let mut wrapper = map.multi_mut(&mut buffer);
        
        let _one = wrapper.get_mut("key_one").unwrap();
        let _two = wrapper.get_mut("key_two").unwrap();
        let _three = wrapper.get_mut("key_three").unwrap();
        let _four = wrapper.get_mut("key_four").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_multi_same_key() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let mut wrapper = map.multi_mut(&mut buffer);
        
        let _one = wrapper.get_mut("key_one").unwrap();
        let _two = wrapper.get_mut("key_two").unwrap();
        let _three = wrapper.get_mut("key_one").unwrap();
    }

    #[test]
    fn test_multi_nonexistent() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let mut wrapper = map.multi_mut(&mut buffer);
        
        assert_eq!(wrapper.get_mut("key_hundred"), None);
    }

    #[test]
    #[should_panic]
    fn test_multi_ref_nonexistent() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let mut wrapper = map.multi_mut(&mut buffer);
        
        wrapper.mut_ref("key_hundred");
    }

    #[test]
    fn test_multi_iter_success() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
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
    }

    #[test]
    fn test_multi_iter_over_capacity() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let keys = ["key_one", "key_two", "key_three"];
        let mut wrapper = map.iter_multi_mut(&keys, &mut buffer);
        
        let _one = wrapper.next().unwrap();
        let _two = wrapper.next().unwrap();
        let _three = wrapper.next().unwrap();

        assert_eq!(wrapper.next(), None);
    }

    #[test]
    #[should_panic]
    fn test_multi_iter_same_key() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let keys = ["key_one", "key_two", "key_one"];
        let mut wrapper = map.iter_multi_mut(&keys, &mut buffer);
        
        let _one = wrapper.next().unwrap();
        let _two = wrapper.next().unwrap();
        let _three = wrapper.next().unwrap();
    }

    #[test]
    fn test_multi_iter_nonexistent() {
        let mut map = populate_hashmap();

        use std::ptr::null;

        let mut buffer = [null(); 3];
        let keys = ["key_hundred"];
        let mut wrapper = map.iter_multi_mut(&keys, &mut buffer);
        
        assert_eq!(wrapper.next(), None);
    }

}
