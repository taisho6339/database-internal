use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::btree::access_manager::AccessManager;
use crate::btree::slotted_page::SlottedPage;

pub struct Node {
    page: Rc<RefCell<SlottedPage>>,
}

impl Node {
    pub fn new(page: Rc<RefCell<SlottedPage>>) -> Self {
        Self {
            page
        }
    }

    pub fn find(&self, key: &[u8]) -> u16 {
        // 二分探索をする関数
        // ■ 中間ノードの場合
        // そのkeyがそのままヒットすればそのIndexを返す
        // keyがヒットしなければそのkeyを挿入できるIndexを返す
        // ■ リーフノードの場合
        // 同じ?

        // ■ 具体的なアルゴリズム
        // pageのpointerの真ん中をとってくる
        // そのpointerが指し示すCellを取得する
        // そのCellのKeyを取得する
        // Cell keyと比較する
        // 同じ => そのIndexを返す
        // 要素数が1 => 大きいならそのIndexの次、小さいならそのIndexの前
        // Cellの方が大きい => 最初の半分で再起する
        // Cellの方が小さい => 後の半分で再起する

        let borrowed_cell: &RefCell<SlottedPage> = self.page.borrow();
        let page_ref = borrowed_cell.borrow();
        let header_view = page_ref.header_view();
        let number_of_pointers = header_view.number_of_pointers().read();

        let mut start = 0;
        let mut end = number_of_pointers - 1;
        loop {
            let mid = (start + end) / 2;
            let pointer = page_ref.borrow().pointer_view(mid as usize);
            let cell_view = page_ref.borrow().cell_view(pointer);
            let cell_key_length = cell_view.borrow().key_length().read() as usize;
            let body = cell_view.borrow().body();
            let cell_key = &body[..cell_key_length];
            let order = key.cmp(cell_key);
            match order {
                Ordering::Equal => {
                    return mid;
                }
                Ordering::Less => {
                    if start == mid {
                        return start;
                    }
                    end = mid;
                    continue;
                }
                Ordering::Greater => {
                    if start == mid {
                        return start + 1;
                    }
                    start = mid;
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use std::io::Write;

    use crate::btree::slotted_page::{cell, MAGIC_NUMBER_LEAF, pointer};

    use super::*;

    // #[test]
    // fn test_find() {
    //     let mut page = SlottedPage::new(MAGIC_NUMBER_LEAF);
    //     page.header_view_mut().number_of_pointers_mut().write(5);
    //     let body = page.body_view();
    //     let pointer_size = pointer::SIZE.unwrap();
    //     for i in 0..5 {
    //         let key = ((i + 1) as u16).to_be_bytes();
    //         let value = (1024 as u16).to_be_bytes();
    //         page.add_cell(i as usize, &key, &value);
    //     }
    //     println!("{:?}", page.body_view());
    //     let node = Node::new(Rc::new(RefCell::new(page)));
    // }
}