use deref_pat::deref_pat;
use pretty_assertions::assert_eq;

#[test]
fn tree() {
    #[derive(Debug)]
    enum Tree {
        Node { left: Box<Tree>, right: Box<Tree> },
        Leaf,
    }

    let tree = Tree::Node {
        left: Tree::Node {
            left: Tree::Leaf.into(),
            right: Tree::Node {
                left: Tree::Leaf.into(),
                right: Tree::Leaf.into(),
            }
            .into(),
        }
        .into(),
        right: Tree::Leaf.into(),
    };

    let matches = deref_pat! {
        if let Tree::Node {
            #[deref]
                left:
                Tree::Node {
                    #[deref]
                        left: leaf @ Tree::Leaf,
                    right: sibling,
                },
                right: other_tree,
        } = tree
        {
            let _ = (other_tree, leaf, sibling);
            true
        } else {
            false
        }
    };
    assert_eq!(matches, true);
}

#[test]
fn vec() {
    struct VecParent {
        vec: Vec<u32>,
    }

    let vec_parent = VecParent { vec: vec![0, 1, 2] };

    deref_pat! {
        if let VecParent { #[deref] vec: [first, second, third] } = &vec_parent {
            assert_eq!(*first, 0);
            assert_eq!(*second, 1);
            assert_eq!(*third, 2);
        } else {
            panic!("pattern did not match");
        }
    };
}
