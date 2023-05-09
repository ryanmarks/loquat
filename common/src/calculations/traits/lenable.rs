use tuple_list::TupleList;

pub trait Lenable {
    fn len(&self) -> i32;
}

impl Lenable for () {
    fn len(&self) -> i32 {
        0
    }
}

impl<Head, Tail: TupleList + Lenable> Lenable for (Head, Tail) {
    fn len(&self) -> i32 {
        1 + self.1.len()
    }
}
