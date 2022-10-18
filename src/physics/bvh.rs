pub mod aabb;
pub mod bounding_sphere;

use crate::math::*;
use generational_arena as ga;

pub use self::{aabb::*, bounding_sphere::*};

pub trait Storage<T> {
    fn store(&mut self, item: T);
}

impl<T> Storage<T> for Vec<T> {
    fn store(&mut self, item: T) {
        self.push(item);
    }
}

pub trait BoundingVolume: Clone + Copy + PartialEq {
    fn intersect(&self, other: &Self) -> bool;
    fn expand(&self, p: &Point3) -> Self {
        let mut ret = self.clone();
        ret.expand_mut(p);
        ret
    }
    fn expand_mut(&mut self, p: &Point3);
    fn merge(&self, p: &Self) -> Self;
    fn volume(&self) -> f32;
}

pub trait BuildBoundingVolume<V: BoundingVolume> {
    fn build_bounding_volume(&self, position: &Isometry3) -> V;
}

pub type BVHLeafId = (ga::Index,);
type BVHInternalId = (ga::Index,);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BVHNodeId {
    Leaf(BVHLeafId),
    Internal(BVHInternalId),
}

#[derive(Debug, Clone, Copy)]
pub struct BVHLeaf<V, T> {
    parent: Option<BVHInternalId>,
    bounding: V,
    data: T,
}
#[derive(Debug, Clone, Copy)]
struct BVHInternal<V> {
    parent: Option<BVHInternalId>,
    left_child: BVHNodeId,
    right_child: BVHNodeId,
    bounding: V,
}

#[derive(Debug)]
pub struct BVH<V, T> {
    root: Option<BVHNodeId>,
    leaves: ga::Arena<BVHLeaf<V, T>>,
    internals: ga::Arena<BVHInternal<V>>,
}

impl<V, T> BVH<V, T> {
    pub fn new() -> Self {
        let root = None;
        let leaves = ga::Arena::new();
        let internals = ga::Arena::new();
        Self {
            root,
            leaves,
            internals,
        }
    }

    fn insert_leaf(&mut self, leaf: BVHLeaf<V, T>) -> BVHLeafId {
        (self.leaves.insert(leaf),)
    }

    fn get_leaf(&self, id: BVHLeafId) -> Option<&BVHLeaf<V, T>> {
        self.leaves.get(id.0)
    }

    fn get_leaf_mut(&mut self, id: BVHLeafId) -> Option<&mut BVHLeaf<V, T>> {
        self.leaves.get_mut(id.0)
    }

    fn remove_leaf(&mut self, id: BVHLeafId) -> Option<BVHLeaf<V, T>> {
        self.leaves.remove(id.0)
    }

    fn insert_internal(&mut self, node: BVHInternal<V>) -> BVHInternalId {
        (self.internals.insert(node),)
    }

    fn get_internal(&self, id: BVHInternalId) -> Option<&BVHInternal<V>> {
        self.internals.get(id.0)
    }
    fn get_internal_mut(&mut self, id: BVHInternalId) -> Option<&mut BVHInternal<V>> {
        self.internals.get_mut(id.0)
    }
    fn remove_internal(&mut self, id: BVHInternalId) -> Option<BVHInternal<V>> {
        self.internals.remove(id.0)
    }
}
impl<V: BoundingVolume, T> BVH<V, T> {
    pub fn insert(&mut self, data: T, bounding: V) -> BVHLeafId {
        match self.root {
            Some(node) => self.insert_at_node(node, data, bounding),
            None => {
                let id = self.insert_leaf(BVHLeaf {
                    parent: None,
                    data,
                    bounding,
                });
                self.root = Some(BVHNodeId::Leaf(id));
                id
            }
        }
    }

    /// Removes a leaf with given id, and returns its data if the leaf exists.
    pub fn remove(&mut self, id: BVHLeafId) -> Option<T> {
        match self.remove_leaf(id) {
            Some(leaf) => match leaf.parent {
                // if leaf is not root node, needs to reconstruct parents.
                Some(parent_id) => {
                    // remove parent.
                    let parent = self.remove_internal(parent_id).unwrap();
                    // get sibling.
                    let sibling_id = if parent.left_child == BVHNodeId::Leaf(id) {
                        parent.right_child
                    } else {
                        parent.left_child
                    };
                    // set sibling's parent to grand parent.
                    let grand_parent_id = match sibling_id {
                        BVHNodeId::Leaf(sibling_id) => {
                            let sibling = self.get_leaf_mut(sibling_id).unwrap();
                            sibling.parent = parent.parent;
                            sibling.parent
                        }
                        BVHNodeId::Internal(sibling_id) => {
                            let sibling = self.get_internal_mut(sibling_id).unwrap();
                            sibling.parent = parent.parent;
                            sibling.parent
                        }
                    };
                    // set grand parent's child node to simbling.
                    match grand_parent_id {
                        Some(grand_parent_id) => {
                            let grand_parent = self.get_internal_mut(grand_parent_id).unwrap();
                            if grand_parent.left_child == BVHNodeId::Internal(parent_id) {
                                grand_parent.left_child = sibling_id;
                            } else {
                                grand_parent.right_child = sibling_id;
                            }
                            self.recalculate_bounding_volume(grand_parent_id);
                        }
                        None => {
                            self.root = Some(sibling_id);
                        }
                    }
                    Some(leaf.data)
                }
                None => {
                    self.root = None;
                    Some(leaf.data)
                }
            },
            None => None,
        }
    }
    fn insert_at_node(&mut self, node: BVHNodeId, data: T, bounding: V) -> BVHLeafId {
        match node {
            BVHNodeId::Leaf(leaf_id) => {
                let new_leaf = BVHLeaf {
                    parent: None,
                    bounding,
                    data,
                };
                let new_leaf_id = self.insert_leaf(new_leaf);
                let leaf = self.get_leaf(leaf_id).unwrap();
                // create new parent internal node.
                let new_internal = BVHInternal {
                    parent: leaf.parent,
                    bounding: leaf.bounding,
                    left_child: BVHNodeId::Leaf(leaf_id),
                    right_child: BVHNodeId::Leaf(new_leaf_id),
                };
                let new_internal_id = self.insert_internal(new_internal);
                // set old parent node's child to new internal node.
                match new_internal.parent {
                    Some(parent_id) => {
                        let parent = self.get_internal_mut(parent_id).unwrap();
                        if parent.left_child == BVHNodeId::Leaf(leaf_id) {
                            parent.left_child = BVHNodeId::Internal(new_internal_id);
                        } else {
                            parent.right_child = BVHNodeId::Internal(new_internal_id);
                        }
                    }
                    None => {
                        self.root = Some(BVHNodeId::Internal(new_internal_id));
                    }
                }
                // set leaf nodes' parent node.
                self.get_leaf_mut(leaf_id).unwrap().parent = Some(new_internal_id);
                self.get_leaf_mut(new_leaf_id).unwrap().parent = Some(new_internal_id);
                self.recalculate_bounding_volume(new_internal_id);
                return new_leaf_id;
            }
            BVHNodeId::Internal(internal_id) => {
                let internal = self.get_internal(internal_id).unwrap();
                let left_bounds = self.get_bounding_volume(internal.left_child).unwrap();
                let right_bounding = self.get_bounding_volume(internal.right_child).unwrap();
                let left_delta = left_bounds.merge(&bounding).volume() - left_bounds.volume();
                let right_delta =
                    right_bounding.merge(&bounding).volume() - right_bounding.volume();
                if left_delta < right_delta {
                    self.insert_at_node(internal.left_child, data, bounding)
                } else {
                    self.insert_at_node(internal.right_child, data, bounding)
                }
            }
        }
    }

    fn recalculate_bounding_volume(&mut self, id: BVHInternalId) {
        let node = self.get_internal(id).unwrap();
        let left_bounding = self.get_bounding_volume(node.left_child).unwrap();
        let right_bounding = self.get_bounding_volume(node.right_child).unwrap();
        let new_bounding = left_bounding.merge(&right_bounding);
        if new_bounding != node.bounding {
            let node_mut = self.get_internal_mut(id).unwrap();
            node_mut.bounding = new_bounding;
            if let Some(parent) = node_mut.parent {
                self.recalculate_bounding_volume(parent)
            }
        }
    }
    fn get_bounding_volume(&self, id: BVHNodeId) -> Option<V> {
        match id {
            BVHNodeId::Leaf(id) => match self.get_leaf(id) {
                Some(l) => Some(l.bounding),
                None => None,
            },
            BVHNodeId::Internal(id) => match self.get_internal(id) {
                Some(i) => Some(i.bounding),
                None => None,
            },
        }
    }
    pub fn get_overlaps<'a, 'b: 'a, S: Storage<(&'a T, &'a T)>>(&'b self, storage: &mut S) {
        match self.root {
            Some(id) => self.get_overlaps_node(id, storage),
            None => (),
        }
    }
    fn get_overlaps_node<'a, 'b: 'a, S: Storage<(&'a T, &'a T)>>(
        &'b self,
        id: BVHNodeId,
        storage: &mut S,
    ) {
        match id {
            BVHNodeId::Leaf(_) => (),
            BVHNodeId::Internal(idx) => {
                let node = self.get_internal(idx).unwrap();
                self.get_overlaps_pair(node.left_child, node.right_child, storage);
            }
        }
    }

    fn get_overlaps_pair<'a, 'b: 'a, S: Storage<(&'a T, &'a T)>>(
        &'b self,
        id1: BVHNodeId,
        id2: BVHNodeId,
        storage: &mut S,
    ) {
        match (id1, id2) {
            (BVHNodeId::Leaf(idx1), BVHNodeId::Leaf(idx2)) => {
                let (l1, l2) = (self.get_leaf(idx1).unwrap(), self.get_leaf(idx2).unwrap());
                if l1.bounding.intersect(&l2.bounding) {
                    storage.store((&l1.data, &l2.data));
                }
            }
            (BVHNodeId::Leaf(lidx), BVHNodeId::Internal(iidx)) => {
                let (leaf, internal) = (
                    self.get_leaf(lidx).unwrap(),
                    self.get_internal(iidx).unwrap(),
                );
                if leaf.bounding.intersect(&internal.bounding) {
                    self.get_overlaps_pair(id1, internal.left_child, storage);
                    self.get_overlaps_pair(id1, internal.right_child, storage);
                }
                self.get_overlaps_pair(internal.left_child, internal.right_child, storage);
            }
            (BVHNodeId::Internal(iidx), BVHNodeId::Leaf(lidx)) => {
                let (leaf, internal) = (
                    self.get_leaf(lidx).unwrap(),
                    self.get_internal(iidx).unwrap(),
                );
                if leaf.bounding.intersect(&internal.bounding) {
                    self.get_overlaps_pair(internal.left_child, id2, storage);
                    self.get_overlaps_pair(internal.right_child, id2, storage);
                }
                self.get_overlaps_pair(internal.left_child, internal.right_child, storage);
            }
            (BVHNodeId::Internal(idx1), BVHNodeId::Internal(idx2)) => {
                let (i1, i2) = (
                    self.get_internal(idx1).unwrap(),
                    self.get_internal(idx2).unwrap(),
                );
                if i1.bounding.intersect(&i2.bounding) {
                    if i1.bounding.volume() > i2.bounding.volume() {
                        self.get_overlaps_pair(i1.left_child, id2, storage);
                        self.get_overlaps_pair(i1.right_child, id2, storage);
                    } else {
                        self.get_overlaps_pair(id1, i2.left_child, storage);
                        self.get_overlaps_pair(id1, i2.right_child, storage);
                    }
                }
                self.get_overlaps_pair(i1.left_child, i1.right_child, storage);
                self.get_overlaps_pair(i2.left_child, i2.right_child, storage);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_bvh() {
        let leaf1 = BVHLeaf {
            parent: None,
            bounding: AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0)),
            data: 0u32,
        };
        let leaf2 = BVHLeaf {
            parent: None,
            bounding: AABB::new(Point3::new(1.5, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0)),
            data: 1u32,
        };
        let mut bvh = BVH::new();
        let l1id = (bvh.leaves.insert(leaf1),);
        let l2id = (bvh.leaves.insert(leaf2),);
        let internal1 = BVHInternal {
            parent: None,
            bounding: leaf1.bounding.merge(&leaf2.bounding),
            left_child: BVHNodeId::Leaf(l1id),
            right_child: BVHNodeId::Leaf(l2id),
        };

        let i1id = bvh.insert_internal(internal1);
        bvh.root = Some(BVHNodeId::Internal(i1id));
        bvh.get_leaf_mut(l1id).unwrap().parent = Some(i1id);
        bvh.get_leaf_mut(l2id).unwrap().parent = Some(i1id);

        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next(), None);
        }

        let leaf3 = BVHLeaf {
            parent: None,
            bounding: AABB::new(Point3::new(4.0, 0.0, 0.0), Vector3::new(0.5, 0.5, 0.5)),
            data: 2,
        };
        let l3id = bvh.insert_leaf(leaf3);
        let internal2 = BVHInternal {
            parent: None,
            bounding: internal1.bounding.merge(&leaf3.bounding),
            left_child: BVHNodeId::Internal(i1id),
            right_child: BVHNodeId::Leaf(l3id),
        };
        let i2id = bvh.insert_internal(internal2);
        bvh.get_leaf_mut(l3id).unwrap().parent = Some(i2id);
        bvh.get_internal_mut(i1id).unwrap().parent = Some(i2id);
        bvh.root = Some(BVHNodeId::Internal(i2id));

        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next(), None);
        }

        let leaf4 = BVHLeaf {
            parent: None,
            bounding: AABB::new(Point3::new(4.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0)),
            data: 3,
        };

        let l4id = bvh.insert_leaf(leaf4);
        let internal3 = BVHInternal {
            parent: Some(i2id),
            bounding: leaf4.bounding.merge(&leaf3.bounding),
            left_child: BVHNodeId::Leaf(l3id),
            right_child: BVHNodeId::Leaf(l4id),
        };
        let i3id = bvh.insert_internal(internal3);
        {
            let i2mut = bvh.get_internal_mut(i2id).unwrap();
            i2mut.right_child = BVHNodeId::Internal(i3id);
            i2mut.bounding = internal1.bounding.merge(&internal3.bounding);
        }
        bvh.get_leaf_mut(l3id).unwrap().parent = Some(i3id);
        bvh.get_leaf_mut(l4id).unwrap().parent = Some(i3id);
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next().unwrap(), &(&2, &3));
            assert_eq!(iter.next(), None);
        }
    }
    #[test]
    fn test_bvh_insert() {
        let mut bvh = BVH::new();
        let leaf1_id = bvh.insert(
            0u32,
            AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0)),
        );
        let leaf2_id = bvh.insert(
            1u32,
            AABB::new(Point3::new(1.5, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0)),
        );

        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next(), None);
        }
        let leaf3_id = bvh.insert(
            2,
            AABB::new(Point3::new(4.0, 0.0, 0.0), Vector3::new(0.5, 0.5, 0.5)),
        );
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next(), None);
        }

        let leaf4_id = bvh.insert(
            3,
            AABB::new(Point3::new(4.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0)),
        );
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next().unwrap(), &(&2, &3));
            assert_eq!(iter.next(), None);
        }

        let leaf2_data = bvh.remove(leaf2_id).unwrap();
        assert_eq!(leaf2_data, 1);
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&2, &3));
            assert_eq!(iter.next(), None);
        }
        let leaf2_id = bvh.insert(
            1u32,
            AABB::new(Point3::new(1.5, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0)),
        );
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&0, &1));
            assert_eq!(iter.next().unwrap(), &(&2, &3));
            assert_eq!(iter.next(), None);
        }

        let leaf1_data = bvh.remove(leaf1_id).unwrap();
        assert_eq!(leaf1_data, 0);
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&2, &3));
            assert_eq!(iter.next(), None);
        }

        let leaf4_data = bvh.remove(leaf4_id).unwrap();
        assert_eq!(leaf4_data, 3);
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next(), None);
        }

        let leaf3_data = bvh.remove(leaf3_id).unwrap();
        assert_eq!(leaf3_data, 2);
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next(), None);
        }
        assert_eq!(bvh.leaves.len(), 1);
        let leaf1_id = bvh.insert(
            0u32,
            AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0)),
        );
        {
            let mut contacts = Vec::new();
            bvh.get_overlaps(&mut contacts);
            let mut iter = contacts.iter();
            assert_eq!(iter.next().unwrap(), &(&1, &0));
            assert_eq!(iter.next(), None);
        }
    }
}
