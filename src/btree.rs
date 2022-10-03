use std::rc::Rc;

use anyhow::Context;
use thiserror::Error;

use crate::access_manager::AccessManager;
use crate::btree::node::Node;
use crate::disk_manager::PageId;

pub mod slotted_page;
pub mod node;

#[derive(Debug, Error)]
pub enum Error {}

struct Btree {
    access_manager: Rc<AccessManager>,
}

const ROOT_PAGE_ID: PageId = PageId(0);

impl Btree {
    pub fn new(access_manager: Rc<AccessManager>) -> Self {
        Self {
            access_manager
        }
    }

    // Insertの実装
    // Branch => Leafの探し当て
    // Leaf => Add cell
    // Pageの使用率が一定を超えたらノードをSplitする
    // 分割はRootノードまで再帰する
    // Pageが溢れたら... => デフラグする...?
    // 左端 or 右端のポインタの扱い

    // TODO:
    // * Splitの条件をもう少し明確にする
    // * Cellのフラグメンテーションの扱いも考慮する
    // * 右端のポインタ含め、Node上のkey:valueの配置を確定させる
    // pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
    //     let mut current_page = self.access_manager.fetch_page(ROOT_PAGE_ID).context("failed to find the root page")?;
    //     let mut node = Node::new(current_page);
    //     loop {
    //         let (index, _) = node.find(key);
    //         if node.is_leaf() {}
    //     }
    //     Ok(())
    // }
}