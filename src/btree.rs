use std::rc::Rc;

use thiserror::Error;

use crate::access_manager::AccessManager;

pub mod slotted_page;
pub mod node;

#[derive(Debug, Error)]
pub enum Error {}

struct Btree {
    access_manager: Rc<AccessManager>,
}

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
    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {

        Ok(())
    }
}