//! Binary Hash Tries, for representing sets and finite maps.
//!
//! Suitable for the Archivist role in Adapton.
//!
// Matthew Hammer <Matthew.Hammer@Colorado.edu>

//use std::rc::Rc;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash,Hasher};
//use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use adapton::engine::*;
use adapton::macros::*;

pub mod simple_level_tree {
    use std::fmt::Debug;
    use std::hash::{Hash};
    use adapton::macros::*;
    use adapton::engine::*;
    use raz::RazTree;
    use raz_meta::Count;

    #[derive(Clone,PartialEq,Eq,Debug,Hash)]
    pub struct BinCons<X> {
        name:Option<Name>,
        level:u32,
        recl:Box<Rec<X>>,
        recr:Box<Rec<X>>
    }
    #[derive(Clone,PartialEq,Eq,Debug,Hash)]
    pub struct LeafCons<X> {
        elms:Vec<X>,
    }
    #[derive(Clone,PartialEq,Eq,Debug,Hash)]
    pub enum Rec<X> {
        Leaf(LeafCons<X>),
        Bin(BinCons<X>),
        Art(Art<Rec<X>>),
    }

    impl<X:'static+Clone+PartialEq+Eq+Debug+Hash> 
        Rec<X> 
    {
        pub fn leaf(xs:Vec<X>) -> Self { 
            Rec::Leaf(LeafCons{elms:xs})
        }
        pub fn bin(opnm:Option<Name>, lev:u32, l:Self, r:Self) -> Self { 
            Rec::Bin(BinCons{name:opnm,level:lev,recl:Box::new(l),recr:Box::new(r)})
        }
        fn art(a:Art<Rec<X>>) -> Self {
            Rec::Art(a)
        }
        pub fn fold_monoid<B:'static+Clone+PartialEq+Eq+Debug+Hash>
            (t:Rec<X>, z:X, b:B, bin:fn(B,X,X)->X, art:fn(Art<X>,X)->X) -> X {
                fn m_leaf<B:Clone,X>(m:(B,fn(B,X,X)->X,X), elms:Vec<X>) -> X { 
                    let mut x = m.2;
                    for elm in elms { x=m.1(m.0.clone(), x, elm) };
                    x
                }
                fn m_bin<B,X>(m:(B,fn(B,X,X)->X,X), _n:Option<Name>, _lev:u32, l:X, r:X) -> X { 
                    m.1(m.0, l, r)
                }
                Self::fold_up::<(B,fn(B,X,X)->X,X),(B,fn(B,X,X)->X,X),X>
                    (t, (b.clone(),bin,z.clone()), m_leaf,
                     (b,bin,z), m_bin, art)
            }
        
        pub fn fold_up
            <L:'static+Clone+PartialEq+Eq+Debug+Hash,
             B:'static+Clone+PartialEq+Eq+Debug+Hash,
             R:'static+Clone+PartialEq+Eq+Debug+Hash>             
            (t:Rec<X>,
             l:L, leaf:fn(L,Vec<X>)->R,
             b:B,  bin:fn(B,Option<Name>,u32,R,R)->R, art:fn(Art<R>,R)->R
            ) -> R 
        {
            match t {
                Rec::Art(a) => Self::fold_up(get!(a), l, leaf, b, bin, art),
                Rec::Leaf(leafcons) => leaf(l, leafcons.elms),
                Rec::Bin(bincons) => {
                    let (n1,n2) = forko!(bincons.name.clone());
                    let res1 = memo!( [n1]? 
                                       Self::fold_up; t:*bincons.recl, 
                                       l:l.clone(), leaf:leaf, b:b.clone(), bin:bin, art:art );
                    let res2 = memo!( [n2]? 
                                       Self::fold_up; t:*bincons.recr,
                                       l:l.clone(), leaf:leaf, b:b.clone(), bin:bin, art:art );
                    let res1 = art(res1.0, res1.1);
                    let res2 = art(res2.0, res2.1);
                    bin(b, bincons.name, bincons.level, res1, res2)
                }
            }
        }
        
        pub fn from_raz_tree(t:RazTree<X,Count>) -> Rec<X> {
            fn at_leaf<X:'static+Clone+PartialEq+Eq+Debug+Hash>
                (v:&Vec<X>) -> Rec<X> {
                    Rec::leaf(v.clone())
                }
            fn at_bin<X:'static+Clone+PartialEq+Eq+Debug+Hash>
                (l:Rec<X>,lev:u32,n:Option<Name>,r:Rec<X>) -> Rec<X> {
                    let (n1,n2) = forko!(n.clone());
                    Rec::bin(n, lev, 
                             Rec::art(cell!([n1]? l)),
                             Rec::art(cell!([n2]? r)))
                }
            t.fold_up_gauged(Rc::new(at_leaf), Rc::new(at_bin)).unwrap()
        }
    }
}

fn my_hash<T>(obj: T) -> HashVal
  where T: Hash
{
  let mut hasher = DefaultHasher::new();
  obj.hash(&mut hasher);
  HashVal(hasher.finish() as usize)
}

/// A hash value -- We define a custom Debug impl for this type.
#[derive(Clone,Hash,Eq,PartialEq)]
pub struct HashVal(usize);

impl fmt::Debug for HashVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.0 & 0b1111)
    }
}

#[derive(PartialEq,Eq,Clone,Hash)]
struct Bits {bits:u32, len:u32}

impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bits{{bits:{:b}, len:{}}}", self.bits, self.len)
    }
}

#[derive(PartialEq,Eq,Clone,Debug,Hash)]
pub struct Trie
    <K:'static+Hash+PartialEq+Eq+Clone+Debug,
     V:'static+Hash+PartialEq+Eq+Clone+Debug>
{
    meta:TrieMeta,
    rec:TrieRec<K,V>,
}

#[derive(PartialEq,Eq,Clone,Debug,Hash)]
pub struct TrieMeta {
    gauge:usize,
}

#[derive(PartialEq,Eq,Clone,Debug,Hash)]
pub enum TrieRec<K:'static+Hash+PartialEq+Eq+Clone+Debug,
             V:'static+Hash+PartialEq+Eq+Clone+Debug>
{
    Empty,
    Leaf(TrieLeaf<K,V>),
    Bin(TrieBin<K,V>),
}
#[derive(PartialEq,Eq,Clone,Debug,Hash)]
pub struct TrieLeaf<K:'static+Hash+PartialEq+Eq+Clone+Debug,
                    V:'static+Hash+PartialEq+Eq+Clone+Debug> {
    kvs:Rc<Vec<(K,HashVal,V)>>,
}
#[derive(Hash,PartialEq,Eq,Clone,Debug)]
pub struct TrieBin<K:'static+Hash+PartialEq+Eq+Clone+Debug,
                   V:'static+Hash+PartialEq+Eq+Clone+Debug> {
    name:   Option<Name>,
    bits:   Bits,
    left:   Art<TrieRec<K,V>>,
    right:  Art<TrieRec<K,V>>,
}

impl<K:'static+Hash+PartialEq+Eq+Clone+Debug,
     V:'static+Hash+PartialEq+Eq+Clone+Debug> Trie<K,V> {

    pub fn find (&self, k:&K) -> Option<V> {
        Self::find_hash(self, my_hash(k), k)
    }

    pub fn find_hash (t: &Trie<K,V>, h:HashVal, k:&K) -> Option<V> {
        Self::find_rec(t, &t.rec, h.clone(), h, k)
    }

    fn find_rec (t: &Trie<K,V>, r:&TrieRec<K,V>, h:HashVal, h_rest:HashVal, k:&K) -> Option<V> {
        match r {
            &TrieRec::Empty => None,
            &TrieRec::Leaf(ref l) => {
                let mut ans = None;
                for &(ref k2,ref k2_hash,ref v) in l.kvs.iter() {
                    if k2_hash == &h && k2 == k {
                        ans = Some(v.clone())
                    }
                }
                return ans
            },
            &TrieRec::Bin(ref b) => {
                if h_rest.0 & 1 == 0 {
                    Self::find_rec(t, &get!(b.left), h, HashVal(h_rest.0 >> 1), k)
                } else { 
                    Self::find_rec(t, &get!(b.right), h, HashVal(h_rest.0 >> 1), k)
                }
            }
        }
    }

    fn split_vec (vec: Rc<Vec<(K,HashVal,V)>>,
                  bits_len:u32,
                  mut vec0:Vec<(K,HashVal,V)>, 
                  mut vec1:Vec<(K,HashVal,V)>)
                  -> (Vec<(K,HashVal,V)>, Vec<(K,HashVal,V)>)
    {
        //let mask : u64 = make_mask(bits_len as usize) as u64;
        for &(ref k, ref k_hash, ref v) in vec.iter() {
            //assert_eq!((mask & k_hash) >> 1, bits.bits as u64); // XXX/???
            if 0 == (k_hash.0 & (1 << bits_len)) {
                vec0.push((k.clone(),k_hash.clone(),v.clone()))
            } else {
                vec1.push((k.clone(),k_hash.clone(),v.clone()))
            }
        };
        (vec0, vec1)
    }

    fn meta (gauge:usize) -> TrieMeta {
        TrieMeta{gauge:gauge}
    }

    pub fn empty (gauge:usize) -> Self { 
        Trie{meta:Self::meta(gauge), rec:TrieRec::Empty}
    }

    pub fn from_vec(vec_in:&Vec<(K,V)>) -> Self { 
        let mut vec = Vec::new();
        for &(ref k, ref v) in vec_in.iter() {
            let k_hash = my_hash(k);
            vec.push((k.clone(),k_hash,v.clone()));
        };
        Trie{meta:Self::meta(vec.len()), 
             rec:TrieRec::Leaf(TrieLeaf{kvs:Rc::new(vec)})}
    }

    pub fn from_key_vec_ref(vec_in:&Vec<K>) -> Trie<K,()> { 
        let mut vec = Vec::new();
        for k in vec_in.iter() {
            let k_hash = my_hash(k);
            vec.push((k.clone(),k_hash,()));
        };
        Trie{meta:Self::meta(vec.len()), 
             rec:TrieRec::Leaf(TrieLeaf{kvs:Rc::new(vec)})}
    }

    pub fn from_key_vec(vec_in:Vec<K>) -> Trie<K,()> { 
        let mut vec = Vec::new();
        for k in vec_in {
            let k_hash = my_hash(&k);
            vec.push((k,k_hash,()));
        };
        Trie{meta:Self::meta(vec.len()), 
             rec:TrieRec::Leaf(TrieLeaf{kvs:Rc::new(vec)})}
    }

    fn split_bits (bits:&Bits) -> (Bits, Bits) {
        let lbits = Bits{len:bits.len+1, bits:/* zero ------ */ bits.bits };
        let rbits = Bits{len:bits.len+1, bits:(1 << bits.len) | bits.bits };
        (lbits, rbits)
    }

    // TODO-Soon: Opt: After splitting a vec, create leaves by first checking whether the vec is empty.

    fn leaf_or_empty (kvs:Vec<(K,HashVal,V)>) -> TrieRec<K,V> {
        if kvs.len() == 0 { TrieRec::Empty }
        else { TrieRec::Leaf(TrieLeaf{kvs:Rc::new(kvs)}) }
    }

    fn is_wf_rec (t:&TrieRec<K,V>, bits:Bits) -> bool {
        match *t {
            TrieRec::Empty => true,
            TrieRec::Leaf(ref leaf) => {
                // Check that all of the hash values match the given bit pattern of bits.
                for &(_,ref hv, _) in leaf.kvs.iter() {
                    for i in 0..bits.len {
                        if ((bits.bits & (1 << i)) as usize) == (hv.0 & (1 << i)) { continue }
                        else { return false }
                    }
                }
                return true
            }
            // Check bit patterns match, and that recursive trees are well-formed.
            TrieRec::Bin(ref b) => { 
                let (b0, b1) = Self::split_bits(&bits);
                let lwf = Self::is_wf_rec(&get!(b.left), b0);
                let rwf = Self::is_wf_rec(&get!(b.right), b1);
                b.bits == bits && lwf && rwf 
            }
        }
    }

    fn is_wf(self:&Self) -> bool {
        Self::is_wf_rec(&self.rec, Bits{bits:0, len:0})
    }

    pub fn join (n:Option<Name>, lt: Self, rt: Self) -> Self {
        //assert_eq!(lt.gauge, rt.gauge); // ??? -- Or take the min? Or the max? Or the average?
        let gauge = if lt.meta.gauge > rt.meta.gauge { lt.meta.gauge } else { rt.meta.gauge };
        Trie{rec:Self::join_rec(TrieMeta{gauge:gauge}, n, lt.rec, rt.rec, Bits{len:0, bits:0}),..lt}
    }

    fn join_rec (meta:TrieMeta, n:Option<Name>, lt: TrieRec<K,V>, rt: TrieRec<K,V>, bits:Bits) -> TrieRec<K,V> {
        match (lt, rt) {
            (TrieRec::Empty,   TrieRec::Empty)   => TrieRec::Empty,
            (TrieRec::Empty,   TrieRec::Leaf(r)) => TrieRec::Leaf(r),
            (TrieRec::Leaf(l), TrieRec::Empty  ) => TrieRec::Leaf(l),
            (TrieRec::Leaf(l), TrieRec::Leaf(r)) => {
                if l.kvs.len() == 0 { 
                    TrieRec::Leaf(r)
                } else if r.kvs.len() == 0 {
                    TrieRec::Leaf(l)
                } else if l.kvs.len() + r.kvs.len() < meta.gauge {
                    // Sub-Case: the leaves, when combined, are smaller than the gauge.
                    let mut vec = (*l.kvs).clone();
                    for &(ref k, ref k_hash, ref v) in r.kvs.iter() { 
                        vec.push((k.clone(),k_hash.clone(),v.clone()));
                    }
                    Self::leaf_or_empty(vec)
                } else {
                    // Sub-Case: the leaves are large enough to justify not being combined.
                    let (e0, e1) = (Vec::new(), Vec::new());
                    let (l0, l1) = Self::split_vec(l.kvs, bits.len, e0, e1);
                    let (r0, r1) = Self::split_vec(r.kvs, bits.len, l0, l1);
                    let (n1, n2) = forko!{n.clone()};
                    let t0 = cell!([n1]? TrieRec::Leaf(TrieLeaf{kvs:Rc::new(r0)}));
                    let t1 = cell!([n2]? TrieRec::Leaf(TrieLeaf{kvs:Rc::new(r1)}));
                    TrieRec::Bin(TrieBin{left:t0, right:t1, bits:bits, name:n})
                }
            },
            (TrieRec::Empty, TrieRec::Bin(r)) => {
                let (b0, b1) = Self::split_bits(&bits);
                let (n0, n1) = forko!{n.clone()};
                let (m0, m1) = forko!{r.name.clone()};
                let o0 = memo!([n0]? Self::join_rec; m:meta.clone(), n:m0, l:TrieRec::Empty, r:get!(r.left), b:b0);
                let o1 = memo!([n1]? Self::join_rec; m:meta.clone(), n:m1, l:TrieRec::Empty, r:get!(r.right), b:b1);
                TrieRec::Bin(TrieBin{ left:o0.0, right:o1.0, name:n, bits:bits })
            },
            (TrieRec::Leaf(l), TrieRec::Bin(r)) => {
                let (e0, e1) = (Vec::new(), Vec::new());
                let (l0, l1) = Self::split_vec(l.kvs, bits.len, e0, e1);
                let (b0, b1) = Self::split_bits(&bits);
                let (n0, n1) = forko!{n.clone()};
                let (m0, m1) = forko!{r.name.clone()};
                let o0 = memo!([n0]? Self::join_rec; m:meta.clone(), n:m0, l:Self::leaf_or_empty(l0), r:get!(r.left),  b:b0);
                let o1 = memo!([n1]? Self::join_rec; m:meta.clone(), n:m1, l:Self::leaf_or_empty(l1), r:get!(r.right), b:b1);
                TrieRec::Bin(TrieBin{ left:o0.0, right:o1.0, name:n, bits:bits })
            },
            (TrieRec::Bin(l), TrieRec::Empty) => {
                let (b0, b1) = Self::split_bits(&bits);
                let (n0, n1) = forko!{n.clone()};
                let (m0, m1) = forko!{l.name.clone()};
                let o0 = memo!([n0]? Self::join_rec; m:meta.clone(), n:m0, l:get!(l.left),  r:TrieRec::Empty, b:b0);
                let o1 = memo!([n1]? Self::join_rec; m:meta.clone(), n:m1, l:get!(l.right), r:TrieRec::Empty, b:b1);
                TrieRec::Bin(TrieBin{ left:o0.0, right:o1.0, name:n, bits:bits })
            },
            (TrieRec::Bin(l), TrieRec::Leaf(r)) => {
                let (e0, e1) = (Vec::new(), Vec::new());
                let (r0, r1) = Self::split_vec(r.kvs, bits.len, e0, e1);
                let (b0, b1) = Self::split_bits(&bits);
                let (n0, n1) = forko!{n.clone()};
                let (m0, m1) = forko!{l.name.clone()};
                let o0 = memo!([n0]? Self::join_rec; m:meta.clone(), n:m0, l:get!(l.left),  r:Self::leaf_or_empty(r0), b:b0);
                let o1 = memo!([n1]? Self::join_rec; m:meta.clone(), n:m1, l:get!(l.right), r:Self::leaf_or_empty(r1), b:b1);
                TrieRec::Bin(TrieBin{ left:o0.0, right:o1.0, name:n, bits:bits })
            },
            (TrieRec::Bin(l), TrieRec::Bin(r)) => {
                let test1 = l.bits == bits;
                let test2 = l.bits == r.bits;
                if !(test1 && test2) {
                    panic!("\nInternal error: {:?} {:?} -- bits:{:?} l.bits:{:?} r.bits:{:?}!!!\n", test1, test2, bits, l.bits, r.bits);
                };
                let (n1, n2) = forko!{n.clone()};
                let (b0, b1) = Self::split_bits(&bits);
                let o0 = memo!([n1]? Self::join_rec; m:meta.clone(), n:l.name, l:get!(l.left),  r:get!(r.left), b:b0);
                let o1 = memo!([n2]? Self::join_rec; m:meta.clone(), n:r.name, l:get!(l.right), r:get!(r.right), b:b1);
                TrieRec::Bin(TrieBin{ left:o0.0, right:o1.0, name:n, bits:bits })
            }
        }
    }
}

#[test] pub fn test_join_10_1   () { test_join(10,1) }
#[test] pub fn test_join_100_1  () { test_join(100,1) }
#[test] pub fn test_join_1000_1 () { test_join(1000,1) }

#[test] pub fn test_join_10_2   () { test_join(10,2) }
#[test] pub fn test_join_100_2  () { test_join(100,2) }
#[test] pub fn test_join_1000_2 () { test_join(1000,2) }

#[test] pub fn test_join_10_3   () { test_join(10,3) }
#[test] pub fn test_join_100_3  () { test_join(100,3) }
#[test] pub fn test_join_1000_3 () { test_join(1000,3) }

#[test] pub fn test_join_10_4   () { test_join(10,4) }
#[test] pub fn test_join_100_4  () { test_join(100,4) }
#[test] pub fn test_join_1000_4 () { test_join(1000,4) }

#[test] pub fn test_join_10_5   () { test_join(10,5) }
#[test] pub fn test_join_100_5  () { test_join(100,5) }
#[test] pub fn test_join_1000_5 () { test_join(1000,5) }


pub fn test_join (size:usize, gauge:usize) {
    use rand::{thread_rng,Rng};
    use adapton::engine::*;
    use archive_stack::*;
    use raz::*;
    use memo::*;
    use level_tree::*;
    use raz_meta::Count;
    use self::simple_level_tree::Rec;

    manage::init_dcg();

    let (elmv,lev_tree) = {
        let mut rng = thread_rng();
        let mut elms : AStack<usize,_> = AStack::new();
        let mut elmv : Vec<usize> = vec![];
        for i in 0..size {
            let elm = rng.gen::<usize>() % size;
            elmv.push(elm);
            elms.push(elm);
            if i % gauge == 0 {
                elms.archive(Some(name_of_usize(i)), gen_branch_level(&mut rng));
            }
        }
        let raz_tree: RazTree<_,Count> = 
            ns( name_of_str("tree_of_stack"), || 
                RazTree::memo_from(&AtHead(elms) ) );

        let lev_tree: Rec<_> = 
            ns( name_of_str("lev_tree_of_raz_tree"), || 
                Rec::from_raz_tree(raz_tree) );

        (elmv,lev_tree)
    };

    fn at_leaf(_:(), v:Vec<usize>) -> Trie<usize,()> {
        Trie::<usize,()>::from_key_vec(v)
    }    
    fn at_bin(_:(),n:Option<Name>,_lev:u32,l:Trie<usize,()>,r:Trie<usize,()>) -> Trie<usize,()> {
        assert!(l.is_wf());
        assert!(r.is_wf());
        ns(n.clone().unwrap(), || Trie::join(n,l,r) )
    }
    fn at_art(_a:Art<Trie<usize,()>>, t:Trie<usize,()>) -> Trie<usize,()> {
        t 
    }
    let trie = ns( name_of_str("trie_of_lev_tree"),
                   || Rec::fold_up( lev_tree,
                                    (), at_leaf,
                                    (), at_bin, 
                                    at_art ) );
    println!("{:?}\n", trie);

    for i in elmv {
        println!("find {:?}", i);
        assert_eq!(trie.find(&i), Some(()));
    }
}