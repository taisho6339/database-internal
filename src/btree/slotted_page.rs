use binary_layout::define_layout;

/*
 4KiB per a page
 -------------------------------------------------------------------
 |                        MagicNumber(4b)                          |
 -------------------------------------------------------------------
 |                     Number of pointers(4b)                      |
 -------------------------------------------------------------------
 |                     Next overflow page id (4b)                  |
 -------------------------------------------------------------------
 |                     Cell offset (4b)                            |
 -------------------------------------------------------------------
 |                        Check sum (4b)                           |
 -------------------------------------------------------------------
 |  Version (1b)  |                  Pointers                      |
 -------------------------------------------------------------------
 |                           Cells                                 |
 -------------------------------------------------------------------
 */

// 4KiB
pub const PAGE_SIZE: usize = 1024 * 4;
pub const HEADER_SIZE: usize = 21;
pub const PAGE_VERSION_V1: u8 = 1;
// 0x32DD is a prefix which represents a page
pub const MAGIC_NUMBER_LEAF: u32 = 0x32DD56AA;
pub const MAGIC_NUMBER_INTERNAL: u32 = 0x32DD77AB;

define_layout!(page_header, BigEndian, {
    magic_number: u32,
    number_of_pointers: u32,
    next_overflow_page_id: u32,
    cell_offset: u32,
    check_sum: u32,
    version: u8,
});

define_layout!(page, BigEndian, {
    header: page_header::NestedView,
    body: [u8; PAGE_SIZE - HEADER_SIZE],
});

#[derive(Debug)]
pub struct SlottedPage {
    data: [u8; PAGE_SIZE],
}

impl SlottedPage {
    pub fn new(magic_number: u32) -> Self {
        let mut s = Self {
            data: [0; PAGE_SIZE]
        };
        let sum = s.check_sum();
        s.header_view_mut().magic_number_mut().write(magic_number);
        s.header_view_mut().version_mut().write(PAGE_VERSION_V1);
        s.header_view_mut().check_sum_mut().write(sum);

        s
    }

    pub fn wrap(data: [u8; PAGE_SIZE]) -> Self {
        Self {
            data
        }
    }

    pub fn empty(&self) -> bool {
        let m = self.header_view().magic_number().read();
        return m != MAGIC_NUMBER_INTERNAL && m != MAGIC_NUMBER_LEAF;
    }

    // Postgres
    // https://github.com/postgres/postgres/blob/2cd2569c72b8920048e35c31c9be30a6170e1410/src/include/storage/checksum_impl.h#L196
    pub fn check_sum(&mut self) -> u32 {
        let old_csum = self.header_view().check_sum().read();

        // Derived from PostgresSQL implementation
        // https://github.com/postgres/postgres/blob/2cd2569c72b8920048e35c31c9be30a6170e1410/src/include/storage/checksum_impl.h#L196
        self.header_view_mut().check_sum_mut().write(0);
        let csum = crc32fast::hash(&self.data);
        self.header_view_mut().check_sum_mut().write(old_csum);

        csum
    }

    pub fn header_view(&self) -> page_header::View<impl AsRef<[u8]> + '_> {
        let page_view = page::View::new(&self.data[..]);
        page_view.into_header()
    }

    pub fn header_view_mut(&mut self) -> page_header::View<impl AsRef<[u8]> + AsMut<[u8]> + '_> {
        let page_view = page::View::new(&mut self.data[..]);
        page_view.into_header()
    }

    pub fn body_view(&self) -> page::View<impl AsRef<[u8]> + '_> {
        page::View::new(&self.data)
    }

    pub fn body_view_mut(&mut self) -> page::View<impl AsRef<[u8]> + '_> {
        page::View::new(&mut self.data)
    }

    pub fn to_bytes(&self) -> &[u8; PAGE_SIZE] {
        page::View::new(&self.data).into_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_as_expected() {
        let page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        assert_eq!(page.header_view().magic_number().read(), MAGIC_NUMBER_LEAF);
        assert_eq!(page.header_view().version().read(), PAGE_VERSION_V1);
        assert_eq!(page.header_view().check_sum().read(), 3340501009);
        assert_eq!(page.header_view().next_overflow_page_id().read(), 0);
        assert_eq!(page.header_view().number_of_pointers().read(), 0);
        assert_eq!(page.header_view().cell_offset().read(), 0);
    }
}

/*
TODO:
    * CellのInsertどうやって実現する？どこにinsertする？insertできるかの判定は？pointerのソートは？free cellの管理は？
    * Vecにするタイミングでコピー発生しちゃうかも？
    * Pageを定義してserialize、deserializeできるようにする
    * 実際にファイルに書き込み読み込みできるようにする
    * Pageをつなげて、探索ができるようにする
    * Addできるようにする
    * 削除ができるようにする
    * 更新ができるようにする
 */
