# Specifications(TBD)

## Data layouts

### Magic Number

* MagicNumber identifies Page or Header?, Root or Leaf or Internal?
    * Page specific bytes -> Page type specific bytes

### Slotted Page Layout

* Page 4096bytes
    * Not saved into disk, but in-memory
        * Page ID
        * Free Cell Table
    * Header
        * EncodeVersion 1bytes
        * MagicNumber 4bytes
        * Pointers size 4bytes
        * Next Overflow page pointer 4bytes
        * CheckSum 4bytes
    * Pointers
        * 4bytes Vector
    * Cells
        * size of a cell is flexible but a page should have fixed number of cells according to the
          B-Tree

* Free Cell Table
    * Manage the array holds an offset in a page and free bytes from it
    * When you remove a cell, the record will be added to this table
    * When you add a cell in the page, you look up enough space from the table first

### Cell Layout

* Key Cell for internal&root nodes
    * key_size 4bytes
    * next_page_id 4bytes
    * key_bytes
* Key-Value Cell for leaf nodes
    * key_size 4bytes
    * value_size 4bytes
    * key_bytes
    * value_bytes

### Considerations

* Use node high key or rightmost pointer
* Checksum for each cell or only all over a Page
* Use sibling pointer or not

## CRUD

### Insert

* Identify a leaf node to insert to
* Insert data into the leaf node = Add cell in the page with one of following patterns
    * No action required 
    * Exceed the size limit of a page => Overflow page
    * Exceed the limit of the number of cells => Node Split of Re-balancing

### Update
TBD

### Delete


### Vacuum and Maintenance
TBD