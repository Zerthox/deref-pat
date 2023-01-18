use deref_pat::deref_pat;
use pretty_assertions::assert_eq;

#[derive(Debug)]
enum Tree {
    Node { left: Box<Tree>, right: Box<Tree> },
    Leaf,
}

fn tree() -> Tree {
    Tree::Node {
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
    }
}

#[test]
fn if_let_expr() {
    let tree = tree();
    let x = deref_pat! {
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
    assert_eq!(x, true);
}
